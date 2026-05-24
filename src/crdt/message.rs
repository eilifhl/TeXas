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
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .context("failed to serialize CRDT message")
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let (message, _) = bincode::serde::decode_from_slice(data, bincode::config::standard())
            .context("failed to deserialize CRDT message")?;
        Ok(message)
    }
}

#[test]
fn crdt_message_round_trips_through_bytes() {
    let sender_id = Uuid::from_u128(2);
    let mut text_crdt = crate::crdt::text_crdt::TextCrdt::new(sender_id);
    let message = CrdtMessage {
        document_id: Uuid::from_u128(1),
        sender_id,
        operation: CrdtOperation::Text(text_crdt.insert(0, 'A')),
    };

    let bytes = message.to_bytes().expect("message should serialize");
    let decoded = CrdtMessage::from_bytes(&bytes).expect("message should deserialize");

    assert_eq!(decoded.document_id, message.document_id);
    assert_eq!(decoded.sender_id, message.sender_id);
    assert!(matches!(
        (decoded.operation, message.operation),
        (
            CrdtOperation::Text(TextOperation::Insert {
                left_neighbor: None,
                right_neighbor: None,
                value: 'A',
                ..
            }),
            CrdtOperation::Text(TextOperation::Insert {
                left_neighbor: None,
                right_neighbor: None,
                value: 'A',
                ..
            })
        )
    ));
}
