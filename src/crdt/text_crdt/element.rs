use uuid::Uuid;
use crate::crdt::text_crdt::Position;
use crate::crdt::text_crdt::timestamp::Timestamp;

pub struct TextElement {
    id: ElementId,
    position: Position,
    value: char, // the text char itself
    deleted: bool,
}

// if replica_id = 'A'
// Insert 'H' -> clock becomes 1 -> element id = (A, 1)
// Insert 'i' -> clock becomes 2 -> element id = (A, 2)
#[derive(Clone, Debug)]
pub struct ElementId {
    replica_id: Uuid,
    counter: Timestamp,
}