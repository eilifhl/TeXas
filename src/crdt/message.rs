use crate::crdt::text_crdt::TextOperation;
use uuid::Uuid;

/// Network envelope for a CRDT operation.
///
/// `CrdtMessage` wraps a concrete CRDT operation with metadata needed by
/// peers to route and apply it to the correct replicated document.
#[derive(Debug, Clone)]
pub struct CrdtMessage {
    pub document_id: Uuid,
    pub sender_id: Uuid,
    pub operation: CrdtOperation,
}

#[derive(Debug, Clone)]
pub enum CrdtOperation {
    Text(TextOperation),
    // later we can add other CRDT operations, i.e., for document labels
}
