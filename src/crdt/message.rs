use crate::crdt::text_crdt::TextOperation;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Network envelope for a CRDT operation.
///
/// `CrdtMessage` wraps a concrete CRDT operation with metadata needed by
/// peers to route and apply it to the correct replicated document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtMessage {
    pub document_id: Uuid,
    pub sender_id: Uuid,
    pub operation: CrdtOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    Text(TextOperation),
    // later we can add other CRDT operations, i.e., for document labels
}

impl CrdtMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("failed to serialize CRDT message")
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).context("failed to deserialize CRDT message")
    }
}
