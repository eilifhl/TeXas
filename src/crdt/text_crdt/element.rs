use uuid::Uuid;
use crate::crdt::text_crdt::Position;

pub struct TextElement {
    id: ElementId,
    position: Position,
    value: char, // the text char itself
    deleted: bool,
}

// if replica_id = 'A'
// Insert 'H' -> clock becomes 1 -> element id = (A, 1)
// Insert 'i' -> clock becomes 2 -> element id = (A, 2)
pub struct ElementId {
    replica_id: Uuid,
    counter: u64,
}