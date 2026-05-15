use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, OperationId, TextElement, TextOperation};
use std::collections::HashSet;
use uuid::Uuid;

pub struct TextCrdt {
    replica_id: Uuid,
    clock: Timestamp,           // to make the IDs, see `ElementId` struct
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
        let element_id = todo!("Retrieve element_id from index");

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

            TextOperation::Delete { .. } => {
                todo!("Implement")
            }
        }
    }
    pub fn value(&self) -> String {
        todo!("Implement")
    }
}
