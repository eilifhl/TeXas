use crate::crdt::text_crdt::ElementId;
use crate::crdt::text_crdt::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
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

    pub fn replica_id(self) -> Uuid {
        self.replica_id
    }

    pub fn counter(self) -> Timestamp {
        self.counter
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOperation {
    Insert {
        op_id: OperationId,
        element_id: ElementId,
        left_neighbor: Option<ElementId>,
        right_neighbor: Option<ElementId>,
        value: char,
    },
    Delete {
        // set element to tombstone
        op_id: OperationId,
        element_id: ElementId,
    },
}

impl TextOperation {
    pub fn operation_id(&self) -> OperationId {
        match self {
            Self::Insert { op_id, .. } | Self::Delete { op_id, .. } => *op_id,
        }
    }
}
