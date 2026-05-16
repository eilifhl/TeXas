use crate::crdt::text_crdt::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct TextElement {
    id: ElementId,
    left_neighbor: Option<ElementId>,
    right_neighbor: Option<ElementId>,
    value: char, // the text char itself
    deleted: bool,
}

impl TextElement {
    pub fn id(&self) -> ElementId {
        self.id
    }

    pub fn left_neighbor(&self) -> Option<ElementId> {
        self.left_neighbor
    }

    pub fn right_neighbor(&self) -> Option<ElementId> {
        self.right_neighbor
    }
}

impl TextElement {
    pub fn new(
        id: ElementId,
        left_neighbor: Option<ElementId>,
        right_neighbor: Option<ElementId>,
        value: char,
    ) -> Self {
        Self {
            id,
            left_neighbor,
            right_neighbor,
            value,
            deleted: false,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.deleted = true;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn value(&self) -> char {
        self.value
    }
}

// if replica_id = 'A'
// Insert 'H' -> clock becomes 1 -> element id = (A, 1)
// Insert 'i' -> clock becomes 2 -> element id = (A, 2)
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElementId {
    replica_id: Uuid,
    counter: Timestamp,
}

impl ElementId {
    pub fn new(replica_id: Uuid, counter: Timestamp) -> Self {
        Self {
            replica_id,
            counter,
        }
    }
}
