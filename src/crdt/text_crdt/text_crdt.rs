use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, OperationId, TextElement, TextOperation};
use std::collections::{BTreeSet, HashMap, HashSet};
use uuid::Uuid;

pub struct TextCrdt {
    replica_id: Uuid,
    clock: Timestamp,           // to make the IDs. See the `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    seen_operations: HashSet<OperationId>,
    pending_deletes: HashSet<ElementId>,
}

impl TextCrdt {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            clock: Timestamp::zero(),
            elements: Vec::new(),
            seen_operations: HashSet::new(),
            pending_deletes: HashSet::new(),
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

                let mut element =
                    TextElement::new(element_id, left_neighbor, right_neighbor, value);

                if self.pending_deletes.remove(&element_id) {
                    element.mark_deleted();
                }

                self.elements.push(element);
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
        self.ordered_elements()
            .into_iter()
            .filter(|element| !element.deleted())
            .map(|element| element.value())
            .collect()
    }

    fn neighbors_for_insert(&self, index: usize) -> (Option<ElementId>, Option<ElementId>) {
        let mut left_neighbor = None;
        let mut visible_index = 0;

        for element in self
            .ordered_elements()
            .into_iter()
            .filter(|element| !element.deleted())
        {
            if visible_index == index {
                return (left_neighbor, Some(element.id()));
            }

            left_neighbor = Some(element.id());
            visible_index += 1;
        }

        (left_neighbor, None)
    }

    fn element_id_from_index(&self, index: usize) -> Option<ElementId> {
        self.ordered_elements()
            .into_iter()
            .filter(|element| !element.deleted())
            .nth(index)
            .map(|element| element.id())
    }

    fn ordered_elements(&self) -> Vec<&TextElement> {
        let mut children_by_left_neighbor: HashMap<Option<ElementId>, Vec<&TextElement>> =
            HashMap::new();

        for element in &self.elements {
            children_by_left_neighbor
                .entry(element.left_neighbor())
                .or_default()
                .push(element);
        }

        for siblings in children_by_left_neighbor.values_mut() {
            Self::order_siblings(siblings);
        }

        let mut ordered = Vec::with_capacity(self.elements.len());
        let mut visited = HashSet::new();

        Self::append_children(None, &children_by_left_neighbor, &mut visited, &mut ordered);

        let mut orphans: Vec<&TextElement> = self
            .elements
            .iter()
            .filter(|element| !visited.contains(&element.id()))
            .collect();
        orphans.sort_by_key(|element| element.id());

        for orphan in orphans {
            Self::append_element(
                orphan,
                &children_by_left_neighbor,
                &mut visited,
                &mut ordered,
            );
        }

        ordered
    }

    fn append_children<'a>(
        left_neighbor: Option<ElementId>,
        children_by_left_neighbor: &HashMap<Option<ElementId>, Vec<&'a TextElement>>,
        visited: &mut HashSet<ElementId>,
        ordered: &mut Vec<&'a TextElement>,
    ) {
        if let Some(children) = children_by_left_neighbor.get(&left_neighbor) {
            for child in children {
                Self::append_element(child, children_by_left_neighbor, visited, ordered);
            }
        }
    }

    fn append_element<'a>(
        element: &'a TextElement,
        children_by_left_neighbor: &HashMap<Option<ElementId>, Vec<&'a TextElement>>,
        visited: &mut HashSet<ElementId>,
        ordered: &mut Vec<&'a TextElement>,
    ) {
        if !visited.insert(element.id()) {
            return;
        }

        ordered.push(element);
        Self::append_children(
            Some(element.id()),
            children_by_left_neighbor,
            visited,
            ordered,
        );
    }

    fn order_siblings(siblings: &mut Vec<&TextElement>) {
        if siblings.len() < 2 {
            return;
        }

        let sibling_ids: HashSet<ElementId> = siblings.iter().map(|element| element.id()).collect();
        let mut outgoing: HashMap<ElementId, Vec<ElementId>> = HashMap::new();
        let mut incoming_counts: HashMap<ElementId, usize> =
            sibling_ids.iter().map(|id| (*id, 0)).collect();

        for element in siblings.iter() {
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

        siblings.sort_by_key(|element| {
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
    let mut text_crdt = TextCrdt::new(Uuid::new_v4());

    text_crdt.insert(0, 'A');
    text_crdt.insert(1, 'B');
    text_crdt.insert(2, 'C');

    text_crdt.delete(1);

    assert_eq!(text_crdt.value(), "AC");
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
