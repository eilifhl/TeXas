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

        let element_id = ElementId::new(self.replica_id, self.clock);
        let op_id = OperationId::new(self.replica_id, self.clock);

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
        todo!("Implement")
    }

    pub fn apply(&mut self, op: TextOperation) {
        todo!("Implement")
    }
    pub fn value(&self) -> String {
        todo!("Implement")
    }
}
