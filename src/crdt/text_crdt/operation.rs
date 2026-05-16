use serde::{Deserialize, Serialize};
use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::{ElementId, Position};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId {
    replica_id: Uuid,
    counter: Timestamp,
}

impl OperationId {
    pub fn new(replica_id: Uuid, counter: Timestamp) -> Self {
        Self {
            replica_id,
            counter,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOperation {
    Insert {
        op_id: OperationId,
        element_id: ElementId,
        position: Position,
        value: char,
    },
    Delete {
        // set element to tombstone
        op_id: OperationId,
        element_id: ElementId,
    },
}
