use std::collections::HashSet;
use uuid::Uuid;
use crate::crdt::text_crdt::{OperationId, TextElement};

pub struct TextCrdt {
    replica_id: Uuid,
    clock: u64, // to make the IDs, see `ElementId` struct
    elements: Vec<TextElement>, // contains the text elements that create the text
    seen_operations: HashSet<OperationId>,
}