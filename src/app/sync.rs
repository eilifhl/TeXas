use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::crdt::text_crdt::{TextCrdt, TextOperation};

const BOOTSTRAP_SYNC_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct SyncState {
    phase: SyncPhase,
}

enum SyncPhase {
    Bootstrapping {
        deadline: Instant,
        sync_requested: bool,
        buffered_remote_operations: Vec<TextOperation>,
    },
    Ready,
}

impl SyncState {
    pub(super) fn new() -> Self {
        Self {
            phase: SyncPhase::Bootstrapping {
                deadline: Instant::now() + BOOTSTRAP_SYNC_TIMEOUT,
                sync_requested: false,
                buffered_remote_operations: Vec::new(),
            },
        }
    }

    pub(super) fn should_accept_authoritative_sync(&self) -> bool {
        matches!(self.phase, SyncPhase::Bootstrapping { .. })
    }

    pub(super) fn editor_locked(&self) -> bool {
        match &self.phase {
            SyncPhase::Bootstrapping { deadline, .. } => Instant::now() < *deadline,
            SyncPhase::Ready => false,
        }
    }

    pub(super) fn can_serve_sync_requests(&self) -> bool {
        matches!(self.phase, SyncPhase::Ready)
    }

    pub(super) fn should_buffer_remote_operations(&self) -> bool {
        matches!(
            self.phase,
            SyncPhase::Bootstrapping {
                sync_requested: true,
                ..
            }
        )
    }

    pub(super) fn mark_bootstrap_requested(&mut self) {
        if let SyncPhase::Bootstrapping { sync_requested, .. } = &mut self.phase {
            *sync_requested = true;
        }
    }

    pub(super) fn mark_authoritative_sync_applied(&mut self) {
        self.phase = SyncPhase::Ready;
    }

    pub(super) fn mark_local_edit(&mut self) {
        self.phase = SyncPhase::Ready;
    }

    pub(super) fn buffer_remote_operation(&mut self, operation: TextOperation) {
        if let SyncPhase::Bootstrapping {
            buffered_remote_operations,
            ..
        } = &mut self.phase
        {
            buffered_remote_operations.push(operation);
        }
    }

    pub(super) fn take_buffered_remote_operations(&mut self) -> Vec<TextOperation> {
        match &mut self.phase {
            SyncPhase::Bootstrapping {
                buffered_remote_operations,
                ..
            } => std::mem::take(buffered_remote_operations),
            SyncPhase::Ready => Vec::new(),
        }
    }

    pub(super) fn finish_bootstrap_if_timed_out(&mut self) -> Option<Vec<TextOperation>> {
        let phase = std::mem::replace(&mut self.phase, SyncPhase::Ready);

        match phase {
            SyncPhase::Bootstrapping {
                deadline,
                buffered_remote_operations,
                ..
            } if Instant::now() >= deadline => Some(buffered_remote_operations),
            other => {
                self.phase = other;
                None
            }
        }
    }
}

pub(super) fn rebuild_text_crdt(
    replica_id: Uuid,
    authoritative_operations: &[TextOperation],
    buffered_remote_operations: &[TextOperation],
) -> TextCrdt {
    let mut text_crdt = TextCrdt::from_operations(replica_id, authoritative_operations);
    for operation in buffered_remote_operations.iter().cloned() {
        text_crdt.apply(operation);
    }
    text_crdt
}

#[test]
fn bootstrap_sync_replaces_authoritative_operations_and_replays_buffered_operations() {
    let mut remote = TextCrdt::new(Uuid::from_u128(20));
    super::seed_text_crdt(&mut remote, super::default_document_id(), "Hello");
    let authoritative_operations = remote.applied_operations().to_vec();
    let buffered_remote_operation = remote.insert(5, '!');

    let rebuilt = rebuild_text_crdt(
        Uuid::from_u128(30),
        &authoritative_operations,
        std::slice::from_ref(&buffered_remote_operation),
    );

    assert_eq!(rebuilt.value(), remote.value());
}

#[test]
fn timeout_finishes_bootstrap_and_returns_buffered_operations() {
    let mut source = TextCrdt::new(Uuid::from_u128(1));
    let operation = source.insert(0, 'A');
    let mut sync_state = SyncState {
        phase: SyncPhase::Bootstrapping {
            deadline: Instant::now() - Duration::from_secs(1),
            sync_requested: true,
            buffered_remote_operations: vec![operation],
        },
    };

    let buffered_operations = sync_state
        .finish_bootstrap_if_timed_out()
        .expect("bootstrap should time out");

    assert_eq!(buffered_operations.len(), 1);
    assert!(sync_state.can_serve_sync_requests());
    assert!(!sync_state.editor_locked());
}

#[test]
fn local_edit_makes_document_ready_without_buffering() {
    let mut source = TextCrdt::new(Uuid::from_u128(1));
    let operation = source.insert(0, 'A');
    let mut sync_state = SyncState::new();

    sync_state.mark_bootstrap_requested();
    sync_state.buffer_remote_operation(operation);
    sync_state.mark_local_edit();

    assert!(sync_state.can_serve_sync_requests());
    assert!(!sync_state.should_accept_authoritative_sync());
    assert!(sync_state.take_buffered_remote_operations().is_empty());
}
