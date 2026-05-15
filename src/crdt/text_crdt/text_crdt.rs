use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, OperationId, Position, TextElement, TextOperation};
use std::collections::HashSet;
use uuid::Uuid;

pub struct TextCrdt {
    replica_id: Uuid,
    clock: Timestamp,           // to make the IDs. See the `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    seen_operations: HashSet<OperationId>,
}

impl TextCrdt {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            clock: Timestamp::zero(),
            elements: Vec::new(),
            seen_operations: HashSet::new(),
        }
    }

    // Insertions and deletions create TextOperations that we apply locally
    // and send over the network to other peers.
    pub fn insert(&mut self, index: usize, value: char) -> TextOperation {
        self.clock = self.clock.next();

        let op_id = OperationId::new(self.replica_id, self.clock);
        let element_id = ElementId::new(self.replica_id, self.clock);

        let position = self.position_for_insert(index);

        let op = TextOperation::Insert {
            op_id,
            element_id,
            position,
            value,
        };

        self.apply(op.clone());

        op
    }

    pub fn delete(&mut self, index: usize) -> Option<TextOperation> {
        let op_id = OperationId::new(self.replica_id, self.clock);
        let element_id = self.element_id_from_index(index)?;

        let op = TextOperation::Delete { op_id, element_id };

        self.apply(op.clone());

        Some(op)
    }

    pub fn apply(&mut self, op: TextOperation) {
        match op {
            TextOperation::Insert {
                op_id,
                element_id,
                position,
                value,
            } => {
                if self.seen_operations.contains(&op_id) {
                    // already applied operation; return early
                    return;
                }

                self.seen_operations.insert(op_id);

                self.elements
                    .push(TextElement::new(element_id, position, value));

                self.elements.sort_by(|a, b| a.position().cmp(b.position()));
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
                }
            }
        }
    }
    pub fn value(&self) -> String {
        todo!("Implement")
    }

    fn position_for_insert(&self, index: usize) -> Position {
        let visible_elements: Vec<&TextElement> = self
            .elements
            .iter()
            .filter(|element| !element.deleted())
            .collect();

        let left = index
            .checked_sub(1)
            .and_then(|left_index| visible_elements.get(left_index))
            .map(|element| element.position());

        let right = visible_elements
            .get(index)
            .map(|element| element.position());

        Position::between(left, right, self.clock, self.replica_id)
    }

    fn element_id_from_index(&self, index: usize) -> Option<ElementId> {
        self.elements
            .iter()
            .filter(|element| !element.deleted())
            .nth(index)
            .map(|element| element.id())
    }
}
