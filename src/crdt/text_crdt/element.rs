use crate::crdt::text_crdt::timestamp::Timestamp;
use crate::crdt::text_crdt::Position;
use uuid::Uuid;

pub struct TextElement {
    id: ElementId,
    position: Position,
    value: char, // the text char itself
    deleted: bool,
}

impl TextElement {
    pub fn id(&self) -> ElementId {
        self.id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl TextElement {
    pub fn new(id: ElementId, position: Position, value: char) -> Self {
        Self {
            id,
            position,
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
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
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
