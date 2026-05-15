use std::collections::HashSet;
use uuid::Uuid;
use crate::crdt::text_crdt::{OperationId, TextElement, TextOperation};

pub struct TextCrdt {
    replica_id: Uuid,
    clock: u64, // to make the IDs, see `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    seen_operations: HashSet<OperationId>,
}

impl TextCrdt {
    pub fn new(replica_id: Uuid) -> Self {
        Self {
            replica_id,
            clock: 0,
            elements: vec![],
            seen_operations: HashSet::new(),
        }
    }

    pub fn insert(&mut self, index: usize, value: char) -> TextOperation {
        todo!("Implement")
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
