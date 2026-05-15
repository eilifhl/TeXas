use uuid::Uuid;
use crate::crdt::text_crdt::{ElementId, Position};
use crate::crdt::text_crdt::timestamp::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId {
    replica_id: Uuid,
    counter: Timestamp,
}

#[derive(Debug, Clone)]
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