use uuid::Uuid;
use crate::crdt::text_crdt::{ElementId, Position};

pub struct OperationId {
    replica_id: Uuid,
    counter: u64,
}

pub enum TextOperation {
    Insert {
        op_id: OperationId,
        element_id: ElementId,
        position: Position,
        value: char,
    },
    Delete { // set element to tombstone
        op_id: OperationId,
        element_id: ElementId,
    },
}