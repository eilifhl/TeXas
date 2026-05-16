use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, OperationId, TextElement, TextOperation};
use std::cell::{Cell, Ref, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use uuid::Uuid;

struct PendingInsert {
    element_id: ElementId,
    left_neighbor: Option<ElementId>,
    right_neighbor: Option<ElementId>,
    value: char,
}

impl PendingInsert {
    fn new(
        element_id: ElementId,
        left_neighbor: Option<ElementId>,
        right_neighbor: Option<ElementId>,
        value: char,
    ) -> Self {
        Self {
            element_id,
            left_neighbor,
            right_neighbor,
            value,
        }
    }
}

pub struct TextCrdt {
    replica_id: Uuid,
    clock: Timestamp,           // to make the IDs. See the `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    ordered_element_indices: RefCell<Vec<usize>>,
    ordered_elements_dirty: Cell<bool>,
    seen_operations: HashSet<OperationId>,
    pending_inserts: HashMap<ElementId, PendingInsert>,
    pending_deletes: HashSet<ElementId>,
    #[cfg(test)]
    ordered_elements_rebuilds: Cell<usize>,
}

impl TextCrdt {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            clock: Timestamp::zero(),
            elements: Vec::new(),
            ordered_element_indices: RefCell::new(Vec::new()),
            ordered_elements_dirty: Cell::new(true),
            seen_operations: HashSet::new(),
            pending_inserts: HashMap::new(),
            pending_deletes: HashSet::new(),
            #[cfg(test)]
            ordered_elements_rebuilds: Cell::new(0),
        }
    }

    // Insertions and deletions create TextOperations that we apply locally
    // and send over the network to other peers.
    pub fn insert(&mut self, index: usize, value: char) -> TextOperation {
        self.clock = self.clock.next();

        let op_id = OperationId::new(self.replica_id, self.clock);
        let element_id = ElementId::new(self.replica_id, self.clock);

        let (left_neighbor, right_neighbor) = self.neighbors_for_insert(index);

        let op = TextOperation::Insert {
            op_id,
            element_id,
            left_neighbor,
            right_neighbor,
            value,
        };

        self.apply(op.clone());

        op
    }

    pub fn delete(&mut self, index: usize) -> Option<TextOperation> {
        // we do this first so we don't increment clock and add a new operation
        // for an invalid element
        let element_id = self.element_id_from_index(index)?;

        self.clock = self.clock.next();
        let op_id = OperationId::new(self.replica_id, self.clock);

        let op = TextOperation::Delete { op_id, element_id };

        self.apply(op.clone());

        Some(op)
    }

    pub fn apply(&mut self, op: TextOperation) {
        match op {
            TextOperation::Insert {
                op_id,
                element_id,
                left_neighbor,
                right_neighbor,
                value,
            } => {
                self.clock = self.clock.max(op_id.counter());

                if self.seen_operations.contains(&op_id) {
                    // already applied operation; return early
                    return;
                }

                self.seen_operations.insert(op_id);

                if self.element_exists(element_id) || self.pending_inserts.contains_key(&element_id)
                {
                    return;
                }

                let pending_insert =
                    PendingInsert::new(element_id, left_neighbor, right_neighbor, value);

                if !self.left_neighbor_exists(left_neighbor) {
                    self.pending_inserts.insert(element_id, pending_insert);
                    return;
                }

                self.materialize_insert(pending_insert);
                self.materialize_pending_inserts();
            }

            TextOperation::Delete { op_id, element_id } => {
                self.clock = self.clock.max(op_id.counter());

                if self.seen_operations.contains(&op_id) {
                    return;
                }

                self.seen_operations.insert(op_id);

                if let Some(element) = self
                    .elements
                    .iter_mut()
                    .find(|element| element.id() == element_id)
                {
                    element.mark_deleted();
                } else {
                    self.pending_deletes.insert(element_id);
                }
            }
        }
    }

    pub fn value(&self) -> String {
        self.ordered_element_indices()
            .iter()
            .filter_map(|&element_index| {
                let element = &self.elements[element_index];

                (!element.deleted()).then_some(element.value())
            })
            .collect()
    }

    fn neighbors_for_insert(&self, index: usize) -> (Option<ElementId>, Option<ElementId>) {
        let mut left_neighbor = None;
        let mut visible_index = 0;

        for &element_index in self.ordered_element_indices().iter() {
            let element = &self.elements[element_index];

            if element.deleted() {
                continue;
            }

            if visible_index == index {
                return (left_neighbor, Some(element.id()));
            }

            left_neighbor = Some(element.id());
            visible_index += 1;
        }

        (left_neighbor, None)
    }

    fn element_id_from_index(&self, index: usize) -> Option<ElementId> {
        self.ordered_element_indices()
            .iter()
            .filter_map(|&element_index| {
                let element = &self.elements[element_index];

                (!element.deleted()).then_some(element.id())
            })
            .nth(index)
    }

    fn element_exists(&self, element_id: ElementId) -> bool {
        self.elements
            .iter()
            .any(|element| element.id() == element_id)
    }

    fn left_neighbor_exists(&self, left_neighbor: Option<ElementId>) -> bool {
        match left_neighbor {
            Some(element_id) => self.element_exists(element_id),
            None => true,
        }
    }

    fn materialize_insert(&mut self, pending_insert: PendingInsert) {
        let mut element = TextElement::new(
            pending_insert.element_id,
            pending_insert.left_neighbor,
            pending_insert.right_neighbor,
            pending_insert.value,
        );

        if self.pending_deletes.remove(&pending_insert.element_id) {
            element.mark_deleted();
        }

        self.elements.push(element);
        self.invalidate_ordered_elements();
    }

    fn materialize_pending_inserts(&mut self) {
        while let Some(element_id) =
            self.pending_inserts
                .iter()
                .find_map(|(&element_id, pending_insert)| {
                    self.left_neighbor_exists(pending_insert.left_neighbor)
                        .then_some(element_id)
                })
        {
            let pending_insert = self
                .pending_inserts
                .remove(&element_id)
                .expect("pending insert exists");

            if !self.element_exists(pending_insert.element_id) {
                self.materialize_insert(pending_insert);
            }
        }
    }

    fn ordered_element_indices(&self) -> Ref<'_, Vec<usize>> {
        if self.ordered_elements_dirty.get() {
            let ordered_element_indices = self.compute_ordered_element_indices();

            *self.ordered_element_indices.borrow_mut() = ordered_element_indices;
            self.ordered_elements_dirty.set(false);

            #[cfg(test)]
            self.ordered_elements_rebuilds
                .set(self.ordered_elements_rebuilds.get() + 1);
        }

        self.ordered_element_indices.borrow()
    }

    fn invalidate_ordered_elements(&self) {
        self.ordered_elements_dirty.set(true);
    }

    fn compute_ordered_element_indices(&self) -> Vec<usize> {
        let mut children_by_left_neighbor: HashMap<Option<ElementId>, Vec<usize>> = HashMap::new();

        for (element_index, element) in self.elements.iter().enumerate() {
            children_by_left_neighbor
                .entry(element.left_neighbor())
                .or_default()
                .push(element_index);
        }

        for siblings in children_by_left_neighbor.values_mut() {
            self.order_siblings(siblings);
        }

        let mut ordered = Vec::with_capacity(self.elements.len());
        let mut visited = HashSet::new();

        self.append_children(None, &children_by_left_neighbor, &mut visited, &mut ordered);
        ordered
    }

    fn append_children(
        &self,
        left_neighbor: Option<ElementId>,
        children_by_left_neighbor: &HashMap<Option<ElementId>, Vec<usize>>,
        visited: &mut HashSet<ElementId>,
        ordered: &mut Vec<usize>,
    ) {
        if let Some(children) = children_by_left_neighbor.get(&left_neighbor) {
            for &child in children {
                self.append_element(child, children_by_left_neighbor, visited, ordered);
            }
        }
    }

    fn append_element(
        &self,
        element_index: usize,
        children_by_left_neighbor: &HashMap<Option<ElementId>, Vec<usize>>,
        visited: &mut HashSet<ElementId>,
        ordered: &mut Vec<usize>,
    ) {
        let element = &self.elements[element_index];

        if !visited.insert(element.id()) {
            return;
        }

        ordered.push(element_index);
        self.append_children(
            Some(element.id()),
            children_by_left_neighbor,
            visited,
            ordered,
        );
    }

    fn order_siblings(&self, siblings: &mut Vec<usize>) {
        if siblings.len() < 2 {
            return;
        }

        let sibling_ids: HashSet<ElementId> = siblings
            .iter()
            .map(|&element_index| self.elements[element_index].id())
            .collect();
        let mut outgoing: HashMap<ElementId, Vec<ElementId>> = HashMap::new();
        let mut incoming_counts: HashMap<ElementId, usize> =
            sibling_ids.iter().map(|id| (*id, 0)).collect();

        for &element_index in siblings.iter() {
            let element = &self.elements[element_index];
            let id = element.id();

            if let Some(right_neighbor) = element.right_neighbor() {
                if right_neighbor != id && sibling_ids.contains(&right_neighbor) {
                    outgoing.entry(id).or_default().push(right_neighbor);
                    *incoming_counts.entry(right_neighbor).or_default() += 1;
                }
            }
        }

        let mut remaining: BTreeSet<ElementId> = sibling_ids.iter().copied().collect();
        let mut ready: BTreeSet<ElementId> = incoming_counts
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(*id))
            .collect();
        let mut ordered_ids = Vec::with_capacity(siblings.len());

        while !remaining.is_empty() {
            let next = ready
                .pop_first()
                .unwrap_or_else(|| *remaining.iter().next().expect("remaining is not empty"));

            if !remaining.remove(&next) {
                continue;
            }

            ordered_ids.push(next);

            for right_neighbor in outgoing.remove(&next).unwrap_or_default() {
                if let Some(incoming_count) = incoming_counts.get_mut(&right_neighbor) {
                    *incoming_count -= 1;

                    if *incoming_count == 0 && remaining.contains(&right_neighbor) {
                        ready.insert(right_neighbor);
                    }
                }
            }
        }

        let ranks: HashMap<ElementId, usize> = ordered_ids
            .into_iter()
            .enumerate()
            .map(|(rank, id)| (id, rank))
            .collect();

        siblings.sort_by_key(|&element_index| {
            let element = &self.elements[element_index];

            (
                ranks
                    .get(&element.id())
                    .copied()
                    .expect("all siblings receive a rank"),
                element.id(),
            )
        });
    }
}

#[test]
fn text_crdt() {
    let mut text_crdt = TextCrdt::new(Uuid::from_u128(1));
    let mut other = TextCrdt::new(Uuid::from_u128(2));

    let insert_a = text_crdt.insert(0, 'A');
    let insert_b = text_crdt.insert(1, 'B');
    let insert_c = text_crdt.insert(2, 'C');

    let delete_b = text_crdt
        .delete(1)
        .expect("inserted element should be deletable");

    assert_eq!(text_crdt.value(), "AC");

    other.apply(insert_a);
    other.apply(insert_c);
    other.apply(insert_b);
    other.apply(delete_b);
    assert_eq!(other.value(), "AC");
}

#[test]
fn concurrent_insertions_at_the_same_index_stay_in_runs() {
    let mut a = TextCrdt::new(Uuid::from_u128(1));
    let mut b = TextCrdt::new(Uuid::from_u128(2));

    let mut initial = Vec::new();
    for (index, value) in "Hello!".chars().enumerate() {
        initial.push(a.insert(index, value));
    }

    for op in initial {
        b.apply(op);
    }

    let mut alice = Vec::new();
    for (offset, value) in " Alice".chars().enumerate() {
        alice.push(a.insert(5 + offset, value));
    }

    let mut charlie = Vec::new();
    for (offset, value) in " Charlie".chars().enumerate() {
        charlie.push(b.insert(5 + offset, value));
    }

    for op in charlie.clone() {
        a.apply(op);
    }

    for op in alice {
        b.apply(op);
    }

    assert_eq!(a.value(), b.value());
    assert!(a.value().contains(" Alice"));
    assert!(a.value().contains(" Charlie"));
    assert!(!a.value().contains("Al Ciharcliee"));
}

#[test]
fn right_neighbor_keeps_middle_inserts_before_the_original_right_element() {
    let mut text_crdt = TextCrdt::new(Uuid::from_u128(1));

    text_crdt.insert(0, 'A');
    text_crdt.insert(1, 'C');
    text_crdt.insert(1, 'B');

    assert_eq!(text_crdt.value(), "ABC");
}

#[test]
fn delete_before_insert_still_tombstones_the_element() {
    let mut a = TextCrdt::new(Uuid::from_u128(1));
    let mut b = TextCrdt::new(Uuid::from_u128(2));

    let insert = a.insert(0, 'A');
    let delete = a.delete(0).expect("inserted element should be deletable");

    b.apply(delete);
    b.apply(insert);

    assert_eq!(b.value(), "");
}

#[test]
fn insert_with_existing_element_id_is_ignored_even_with_new_operation_id() {
    let mut text_crdt = TextCrdt::new(Uuid::from_u128(1));
    let replica_id = Uuid::from_u128(2);
    let first = Timestamp::zero().next();
    let second = first.next();
    let element_id = ElementId::new(replica_id, first);

    text_crdt.apply(TextOperation::Insert {
        op_id: OperationId::new(replica_id, first),
        element_id,
        left_neighbor: None,
        right_neighbor: None,
        value: 'A',
    });

    text_crdt.apply(TextOperation::Insert {
        op_id: OperationId::new(replica_id, second),
        element_id,
        left_neighbor: None,
        right_neighbor: None,
        value: 'B',
    });

    assert_eq!(text_crdt.elements.len(), 1);
    assert_eq!(text_crdt.value(), "A");
}

#[test]
fn ordered_view_is_cached_between_mutations() {
    let mut text_crdt = TextCrdt::new(Uuid::from_u128(1));

    text_crdt.insert(0, 'A');

    assert_eq!(text_crdt.value(), "A");
    assert_eq!(text_crdt.ordered_elements_rebuilds.get(), 2);

    assert_eq!(text_crdt.value(), "A");
    assert_eq!(text_crdt.ordered_elements_rebuilds.get(), 2);

    text_crdt.insert(1, 'B');
    assert_eq!(text_crdt.value(), "AB");
    assert_eq!(text_crdt.ordered_elements_rebuilds.get(), 3);

    text_crdt.delete(0);
    assert_eq!(text_crdt.value(), "B");
    assert_eq!(text_crdt.ordered_elements_rebuilds.get(), 3);
}

#[test]
fn insert_with_missing_left_neighbor_is_buffered_until_parent_arrives() {
    let replica_id = Uuid::from_u128(1);
    let parent_timestamp = Timestamp::zero().next();
    let child_timestamp = parent_timestamp.next();
    let parent_id = ElementId::new(replica_id, parent_timestamp);
    let child_id = ElementId::new(replica_id, child_timestamp);

    let parent = TextOperation::Insert {
        op_id: OperationId::new(replica_id, parent_timestamp),
        element_id: parent_id,
        left_neighbor: None,
        right_neighbor: None,
        value: 'A',
    };
    let child = TextOperation::Insert {
        op_id: OperationId::new(replica_id, child_timestamp),
        element_id: child_id,
        left_neighbor: Some(parent_id),
        right_neighbor: None,
        value: 'B',
    };

    let mut text_crdt = TextCrdt::new(Uuid::from_u128(2));

    text_crdt.apply(child);

    assert_eq!(text_crdt.elements.len(), 0);
    assert_eq!(text_crdt.pending_inserts.len(), 1);
    assert_eq!(text_crdt.value(), "");

    text_crdt.apply(parent);

    assert_eq!(text_crdt.elements.len(), 2);
    assert_eq!(text_crdt.pending_inserts.len(), 0);
    assert_eq!(text_crdt.value(), "AB");
}

#[test]
fn delete_before_buffered_insert_tombstones_element_when_parent_arrives() {
    let replica_id = Uuid::from_u128(1);
    let parent_timestamp = Timestamp::zero().next();
    let child_timestamp = parent_timestamp.next();
    let delete_timestamp = child_timestamp.next();
    let parent_id = ElementId::new(replica_id, parent_timestamp);
    let child_id = ElementId::new(replica_id, child_timestamp);

    let child = TextOperation::Insert {
        op_id: OperationId::new(replica_id, child_timestamp),
        element_id: child_id,
        left_neighbor: Some(parent_id),
        right_neighbor: None,
        value: 'B',
    };
    let delete_child = TextOperation::Delete {
        op_id: OperationId::new(replica_id, delete_timestamp),
        element_id: child_id,
    };
    let parent = TextOperation::Insert {
        op_id: OperationId::new(replica_id, parent_timestamp),
        element_id: parent_id,
        left_neighbor: None,
        right_neighbor: None,
        value: 'A',
    };

    let mut text_crdt = TextCrdt::new(Uuid::from_u128(2));

    text_crdt.apply(child);
    text_crdt.apply(delete_child);
    text_crdt.apply(parent);

    assert_eq!(text_crdt.elements.len(), 2);
    assert_eq!(text_crdt.pending_inserts.len(), 0);
    assert_eq!(text_crdt.value(), "A");
}

#[test]
fn buffered_insert_chain_materializes_when_root_parent_arrives() {
    let replica_id = Uuid::from_u128(1);
    let first_timestamp = Timestamp::zero().next();
    let second_timestamp = first_timestamp.next();
    let third_timestamp = second_timestamp.next();
    let first_id = ElementId::new(replica_id, first_timestamp);
    let second_id = ElementId::new(replica_id, second_timestamp);
    let third_id = ElementId::new(replica_id, third_timestamp);

    let first = TextOperation::Insert {
        op_id: OperationId::new(replica_id, first_timestamp),
        element_id: first_id,
        left_neighbor: None,
        right_neighbor: None,
        value: 'A',
    };
    let second = TextOperation::Insert {
        op_id: OperationId::new(replica_id, second_timestamp),
        element_id: second_id,
        left_neighbor: Some(first_id),
        right_neighbor: None,
        value: 'B',
    };
    let third = TextOperation::Insert {
        op_id: OperationId::new(replica_id, third_timestamp),
        element_id: third_id,
        left_neighbor: Some(second_id),
        right_neighbor: None,
        value: 'C',
    };

    let mut text_crdt = TextCrdt::new(Uuid::from_u128(2));

    text_crdt.apply(third);
    text_crdt.apply(second);
    assert_eq!(text_crdt.value(), "");

    text_crdt.apply(first);

    assert_eq!(text_crdt.pending_inserts.len(), 0);
    assert_eq!(text_crdt.value(), "ABC");
}

#[test]
fn applying_remote_operation_advances_the_local_clock() {
    let mut a = TextCrdt::new(Uuid::from_u128(1));
    let mut b = TextCrdt::new(Uuid::from_u128(2));

    let first = a.insert(0, 'A');
    let second = a.insert(1, 'B');

    b.apply(first);
    b.apply(second.clone());

    let remote_counter = match second {
        TextOperation::Insert { op_id, .. } => op_id.counter(),
        TextOperation::Delete { .. } => unreachable!("insert returns an insert operation"),
    };

    let local = b.insert(2, 'C');
    let local_counter = match local {
        TextOperation::Insert { op_id, .. } => op_id.counter(),
        TextOperation::Delete { .. } => unreachable!("insert returns an insert operation"),
    };

    assert!(local_counter > remote_counter);
}

#[test]
fn sibling_ordering_honors_transitive_right_neighbor_chains() {
    let replica_id = Uuid::from_u128(1);
    let one = Timestamp::zero().next();
    let two = one.next();
    let three = two.next();

    let a = ElementId::new(replica_id, three);
    let b = ElementId::new(replica_id, two);
    let c = ElementId::new(replica_id, one);

    let operations = [
        TextOperation::Insert {
            op_id: OperationId::new(replica_id, three),
            element_id: a,
            left_neighbor: None,
            right_neighbor: Some(b),
            value: 'A',
        },
        TextOperation::Insert {
            op_id: OperationId::new(replica_id, two),
            element_id: b,
            left_neighbor: None,
            right_neighbor: Some(c),
            value: 'B',
        },
        TextOperation::Insert {
            op_id: OperationId::new(replica_id, one),
            element_id: c,
            left_neighbor: None,
            right_neighbor: None,
            value: 'C',
        },
    ];

    let mut forward = TextCrdt::new(Uuid::from_u128(2));
    let mut reverse = TextCrdt::new(Uuid::from_u128(3));

    for op in operations.iter().cloned() {
        forward.apply(op);
    }

    for op in operations.iter().rev().cloned() {
        reverse.apply(op);
    }

    assert_eq!(forward.value(), "ABC");
    assert_eq!(reverse.value(), "ABC");
}
