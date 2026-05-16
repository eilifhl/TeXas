use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, OperationId, TextElement, TextOperation};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
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
                if self.seen_operations.contains(&op_id) {
                    // already applied operation; return early
                    return;
                }

                self.seen_operations.insert(op_id);

                let mut element = TextElement::new(
                    element_id,
                    left_neighbor,
                    right_neighbor,
                    value,
                );

                if self.pending_deletes.remove(&element_id) {
                    element.mark_deleted();
                }

                self.elements.push(element);
            }

            TextOperation::Delete { op_id, element_id } => {
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

        for children in children_by_left_neighbor.values_mut() {
            children.sort_by(|a, b| Self::compare_siblings(a, b));
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

    fn compare_siblings(a: &TextElement, b: &TextElement) -> Ordering {
        if a.right_neighbor() == Some(b.id()) {
            return Ordering::Less;
        }

        if b.right_neighbor() == Some(a.id()) {
            return Ordering::Greater;
        }

        a.id().cmp(&b.id())
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
