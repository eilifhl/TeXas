pub mod build;
pub mod platform;
pub mod theme;
mod ui;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use eframe::egui::Context;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::crdt::message::{CrdtMessage, CrdtOperation};
use crate::crdt::text_crdt::{OperationKnowledge, TextCrdt, TextOperation};
use crate::network::{self, DEFAULT_DOCUMENT_TOPIC, NetworkEvent, NetworkHandle};
use build::run_latexmk;
use platform::open_path_with_default_app;
use theme::{ThemeMode, configure_theme};

const BOOTSTRAP_SYNC_TIMEOUT: Duration = Duration::from_secs(3);

pub struct TexasApp {
    model: AppModel,
    network: NetworkHandle,
    _runtime: Runtime,
}

struct AppModel {
    document_id: Uuid,
    replica_id: Uuid,
    text_crdt: TextCrdt,
    editor_text: String,
    document_path: PathBuf,
    output_text: String,
    last_pdf_path: Option<PathBuf>,
    theme_mode: ThemeMode,
    network_status: NetworkStatus,
    sync_state: SyncState,
}

#[derive(Clone, Default)]
struct NetworkStatus {
    local_peer_id: Option<String>,
    listen_addr: Option<String>,
    peers: HashSet<String>,
}

struct SyncState {
    bootstrap_deadline: Instant,
    bootstrap_requested: bool,
    authoritative_sync_applied: bool,
    local_state_is_authoritative: bool,
    local_edits_since_startup: bool,
    buffered_remote_operations: Vec<TextOperation>,
}

impl SyncState {
    fn new() -> Self {
        Self {
            bootstrap_deadline: Instant::now() + BOOTSTRAP_SYNC_TIMEOUT,
            bootstrap_requested: false,
            authoritative_sync_applied: false,
            local_state_is_authoritative: false,
            local_edits_since_startup: false,
            buffered_remote_operations: Vec::new(),
        }
    }

    fn should_accept_authoritative_sync(&self) -> bool {
        !self.authoritative_sync_applied
            && !self.local_state_is_authoritative
            && !self.local_edits_since_startup
    }

    fn editor_locked(&self) -> bool {
        self.should_accept_authoritative_sync() && Instant::now() < self.bootstrap_deadline
    }

    fn can_serve_sync_requests(&self) -> bool {
        self.authoritative_sync_applied
            || self.local_state_is_authoritative
            || self.local_edits_since_startup
    }

    fn should_buffer_remote_operations(&self) -> bool {
        self.bootstrap_requested && self.should_accept_authoritative_sync()
    }

    fn mark_bootstrap_requested(&mut self) {
        self.bootstrap_requested = true;
    }

    fn mark_authoritative_sync_applied(&mut self) {
        self.bootstrap_requested = false;
        self.authoritative_sync_applied = true;
        self.buffered_remote_operations.clear();
    }

    fn mark_local_edit(&mut self) {
        self.bootstrap_requested = false;
        self.local_edits_since_startup = true;
        self.local_state_is_authoritative = true;
        self.buffered_remote_operations.clear();
    }

    fn buffer_remote_operation(&mut self, operation: TextOperation) {
        self.buffered_remote_operations.push(operation);
    }

    fn take_buffered_remote_operations(&mut self) -> Vec<TextOperation> {
        std::mem::take(&mut self.buffered_remote_operations)
    }

    fn mark_local_state_authoritative_if_timed_out(&mut self) -> bool {
        if self.should_accept_authoritative_sync() && Instant::now() >= self.bootstrap_deadline {
            self.bootstrap_requested = false;
            self.local_state_is_authoritative = true;
            self.buffered_remote_operations.clear();
            true
        } else {
            false
        }
    }
}

enum AppEffect {
    Log(String),
}

impl AppModel {
    fn new(theme_mode: ThemeMode) -> Result<Self> {
        let document_path = default_document_path();
        let initial_text = load_document(&document_path)?;
        let document_id = default_document_id();
        let replica_id = Uuid::new_v4();
        let mut text_crdt = TextCrdt::new(replica_id);
        seed_text_crdt(&mut text_crdt, document_id, &initial_text);
        let editor_text = text_crdt.value();

        Ok(Self {
            document_id,
            replica_id,
            text_crdt,
            editor_text,
            document_path,
            output_text: "No build output yet.".to_owned(),
            last_pdf_path: None,
            theme_mode,
            network_status: NetworkStatus::default(),
            sync_state: SyncState::new(),
        })
    }

    fn apply_build_result(&mut self, result: build::BuildResult) {
        self.output_text = result.output;
        self.last_pdf_path = result.pdf_path;
    }

    fn apply_network_event(&mut self, event: NetworkEvent) -> Option<AppEffect> {
        let current_status = std::mem::take(&mut self.network_status);
        let (next_status, effect) = reduce_network_event(current_status, event);
        self.network_status = next_status;
        effect
    }

    fn apply_effect(&mut self, effect: AppEffect) {
        match effect {
            AppEffect::Log(line) => {
                self.output_text = append_output_line(&self.output_text, &line);
            }
        }
    }
}

impl TexasApp {
    pub fn new(ctx: &Context, runtime: Runtime) -> Result<Self> {
        let theme_mode = ThemeMode::Dark;
        configure_theme(ctx, theme_mode);
        let repaint_signal = Arc::new({
            let ctx = ctx.clone();
            move || ctx.request_repaint()
        });

        let network = network::start(runtime.handle(), repaint_signal)?;
        network.subscribe(DEFAULT_DOCUMENT_TOPIC)?;

        let mut app = Self {
            model: AppModel::new(theme_mode)?,
            network,
            _runtime: runtime,
        };
        app.request_document_sync("startup");

        Ok(app)
    }

    pub(crate) fn compile(&mut self) {
        let result = run_latexmk(&self.model.editor_text);
        self.model.apply_build_result(result);
    }

    pub(crate) fn persist_document(&mut self) {
        if let Err(error) = save_document(&self.model.document_path, &self.model.editor_text) {
            self.model.apply_effect(AppEffect::Log(format!(
                "[editor] failed to save {}: {error}",
                self.model.document_path.display()
            )));
        }
    }

    pub(crate) fn open_pdf(&mut self) {
        let Some(pdf_path) = &self.model.last_pdf_path else {
            self.model.output_text = append_output_line(
                &self.model.output_text,
                "No compiled PDF available to open.",
            );
            return;
        };

        if let Err(error) = open_path_with_default_app(pdf_path) {
            self.model.output_text = append_output_line(
                &self.model.output_text,
                &format!("Failed to open PDF:\n{}\n{error}", pdf_path.display()),
            );
        }
    }

    pub(crate) fn toggle_theme(&mut self, ctx: &Context) {
        self.model.theme_mode = self.model.theme_mode.toggle();
        configure_theme(ctx, self.model.theme_mode);
    }

    pub(crate) fn apply_local_editor_change(&mut self, previous_text: &str, next_text: &str) {
        let Some(delta) = diff_text(previous_text, next_text) else {
            return;
        };

        self.model.sync_state.mark_local_edit();

        let mut operations = Vec::with_capacity(delta.removed_count + delta.inserted.len());

        for offset in (0..delta.removed_count).rev() {
            let delete_index = delta.prefix_len + offset;
            let Some(operation) = self.model.text_crdt.delete(delete_index) else {
                self.model.apply_effect(AppEffect::Log(format!(
                    "[crdt] failed to delete char at index {delete_index}",
                )));
                return;
            };
            operations.push(operation);
        }

        for (offset, value) in delta.inserted.iter().copied().enumerate() {
            let insert_index = delta.prefix_len + offset;
            operations.push(self.model.text_crdt.insert(insert_index, value));
        }

        self.model.editor_text = self.model.text_crdt.value();
        self.persist_document();

        for operation in operations {
            self.publish_operation(operation);
        }
    }

    pub(crate) fn sync_network_state(&mut self) {
        for event in self.network.poll_events() {
            match event {
                NetworkEvent::MessageReceived { topic, payload } => {
                    self.apply_remote_message(&topic, &payload);
                }
                NetworkEvent::PeerDiscovered(peer_id) => {
                    if self.model.sync_state.should_accept_authoritative_sync() {
                        self.request_document_sync("peer discovered");
                    }

                    if let Some(effect) = self
                        .model
                        .apply_network_event(NetworkEvent::PeerDiscovered(peer_id))
                    {
                        self.model.apply_effect(effect);
                    }
                }
                other => {
                    if let Some(effect) = self.model.apply_network_event(other) {
                        self.model.apply_effect(effect);
                    }
                }
            }
        }
    }

    pub(crate) fn connection_label(&self) -> String {
        let peer = self
            .model
            .network_status
            .local_peer_id
            .as_deref()
            .map(short_peer_id)
            .unwrap_or("starting");

        if let Some(address) = &self.model.network_status.listen_addr {
            format!(
                "Peer {peer} | {} peer(s) | {address}",
                self.model.network_status.peers.len()
            )
        } else {
            format!(
                "Peer {peer} | {} peer(s)",
                self.model.network_status.peers.len()
            )
        }
    }

    pub(crate) fn editor_locked(&self) -> bool {
        self.model.sync_state.editor_locked()
    }

    fn apply_remote_message(&mut self, topic: &str, payload: &[u8]) {
        let message = match CrdtMessage::from_bytes(payload) {
            Ok(message) => message,
            Err(error) => {
                self.model.apply_effect(AppEffect::Log(format!(
                    "[network] ignored invalid CRDT message on {topic}: {error}",
                )));
                return;
            }
        };

        if message.sender_id == self.model.replica_id {
            return;
        }

        if message.document_id != self.model.document_id {
            self.model.apply_effect(AppEffect::Log(format!(
                "[network] ignored message for document {}",
                short_uuid(message.document_id),
            )));
            return;
        }

        match message.operation {
            CrdtOperation::Text(operation) => {
                if self.model.sync_state.should_buffer_remote_operations() {
                    self.model.sync_state.buffer_remote_operation(operation);
                } else {
                    self.apply_text_operation(operation);
                }
            }
            CrdtOperation::SyncRequest { known_operations } => {
                self.respond_to_sync_request(message.sender_id, known_operations);
            }
            CrdtOperation::SyncResponse {
                target_replica_id,
                operations,
            } => {
                if target_replica_id != self.model.replica_id {
                    return;
                }

                self.apply_sync_response(message.sender_id, operations);
            }
        }
    }

    fn publish_operation(&mut self, operation: TextOperation) {
        self.publish_message(
            CrdtMessage {
                document_id: self.model.document_id,
                sender_id: self.model.replica_id,
                operation: CrdtOperation::Text(operation),
            },
            "CRDT operation",
        );
    }

    fn apply_text_operation(&mut self, operation: TextOperation) {
        self.model.text_crdt.apply(operation);
        let next_text = self.model.text_crdt.value();
        if next_text != self.model.editor_text {
            self.model.editor_text = next_text;
            self.persist_document();
        }
    }

    fn request_document_sync(&mut self, reason: &str) {
        if !self.model.sync_state.should_accept_authoritative_sync() {
            return;
        }

        self.model.sync_state.mark_bootstrap_requested();
        self.publish_message(
            CrdtMessage {
                document_id: self.model.document_id,
                sender_id: self.model.replica_id,
                operation: CrdtOperation::SyncRequest {
                    known_operations: OperationKnowledge::default(),
                },
            },
            "document sync request",
        );
        self.model.apply_effect(AppEffect::Log(format!(
            "[sync] requested authoritative document sync ({reason})"
        )));
    }

    fn respond_to_sync_request(
        &mut self,
        requester_id: Uuid,
        known_operations: OperationKnowledge,
    ) {
        if !self.model.sync_state.can_serve_sync_requests() {
            return;
        }

        let operations = self.model.text_crdt.missing_operations(&known_operations);
        let operation_count = operations.len();
        self.publish_message(
            CrdtMessage {
                document_id: self.model.document_id,
                sender_id: self.model.replica_id,
                operation: CrdtOperation::SyncResponse {
                    target_replica_id: requester_id,
                    operations,
                },
            },
            "document sync response",
        );
        self.model.apply_effect(AppEffect::Log(format!(
            "[sync] shared {operation_count} operation(s) with {}",
            short_uuid(requester_id),
        )));
    }

    fn apply_sync_response(&mut self, sender_id: Uuid, operations: Vec<TextOperation>) {
        if !self.model.sync_state.should_accept_authoritative_sync() {
            return;
        }

        let buffered_remote_operations = self.model.sync_state.take_buffered_remote_operations();
        let next_crdt = rebuild_text_crdt(
            self.model.replica_id,
            &operations,
            &buffered_remote_operations,
        );
        let next_text = next_crdt.value();

        self.model.text_crdt = next_crdt;
        self.model.editor_text = next_text;
        self.model.sync_state.mark_authoritative_sync_applied();
        self.persist_document();
        self.model.apply_effect(AppEffect::Log(format!(
            "[sync] applied {} authoritative operation(s) from {} and replayed {} buffered operation(s)",
            operations.len(),
            short_uuid(sender_id),
            buffered_remote_operations.len()
        )));
    }

    fn publish_message(&mut self, message: CrdtMessage, label: &str) {
        let payload = match message.to_bytes() {
            Ok(payload) => payload,
            Err(error) => {
                self.model.apply_effect(AppEffect::Log(format!(
                    "[network] failed to serialize {label}: {error}",
                )));
                return;
            }
        };

        if let Err(error) = self.network.publish(DEFAULT_DOCUMENT_TOPIC, payload) {
            self.model.apply_effect(AppEffect::Log(format!(
                "[network] failed to publish {label}: {error}",
            )));
        }
    }

    fn refresh_sync_state(&mut self) {
        if self
            .model
            .sync_state
            .mark_local_state_authoritative_if_timed_out()
        {
            self.model.apply_effect(AppEffect::Log(
                "[sync] no authoritative peer answered in time; using local document".to_owned(),
            ));
        }
    }
}

impl eframe::App for TexasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.refresh_sync_state();
        self.sync_network_state();
        self.render(ctx);
    }
}

fn append_output_line(output_text: &str, line: &str) -> String {
    if output_text == "No build output yet." {
        line.to_owned()
    } else {
        let mut next = output_text.to_owned();
        if !output_text.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(line);
        next
    }
}

fn short_peer_id(peer_id: &str) -> &str {
    peer_id.get(..12).unwrap_or(peer_id)
}

fn short_uuid(id: Uuid) -> String {
    id.to_string().chars().take(12).collect()
}

fn default_document_id() -> Uuid {
    Uuid::from_u128(1)
}

fn default_document_path() -> PathBuf {
    PathBuf::from("main.tex")
}

fn bootstrap_replica_id(document_id: Uuid) -> Uuid {
    Uuid::from_u128(document_id.as_u128() ^ 0xB00757A7000000000000000000000000)
}

fn seed_text_crdt(text_crdt: &mut TextCrdt, document_id: Uuid, text: &str) {
    let mut bootstrap = TextCrdt::new(bootstrap_replica_id(document_id));

    for (index, value) in text.chars().enumerate() {
        let operation = bootstrap.insert(index, value);
        text_crdt.apply(operation);
    }
}

fn rebuild_text_crdt(
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

struct TextDelta {
    prefix_len: usize,
    removed_count: usize,
    inserted: Vec<char>,
}

fn diff_text(previous_text: &str, next_text: &str) -> Option<TextDelta> {
    let previous_chars: Vec<char> = previous_text.chars().collect();
    let next_chars: Vec<char> = next_text.chars().collect();

    let mut prefix_len = 0;
    while prefix_len < previous_chars.len()
        && prefix_len < next_chars.len()
        && previous_chars[prefix_len] == next_chars[prefix_len]
    {
        prefix_len += 1;
    }

    let mut previous_suffix_start = previous_chars.len();
    let mut next_suffix_start = next_chars.len();
    while previous_suffix_start > prefix_len
        && next_suffix_start > prefix_len
        && previous_chars[previous_suffix_start - 1] == next_chars[next_suffix_start - 1]
    {
        previous_suffix_start -= 1;
        next_suffix_start -= 1;
    }

    let removed_count = previous_suffix_start - prefix_len;
    let inserted = next_chars[prefix_len..next_suffix_start].to_vec();

    if removed_count == 0 && inserted.is_empty() {
        None
    } else {
        Some(TextDelta {
            prefix_len,
            removed_count,
            inserted,
        })
    }
}

fn load_document(path: &Path) -> std::io::Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    }
}

fn save_document(path: &Path, contents: &str) -> std::io::Result<()> {
    fs::write(path, contents)
}

fn reduce_network_event(
    mut status: NetworkStatus,
    event: NetworkEvent,
) -> (NetworkStatus, Option<AppEffect>) {
    match event {
        NetworkEvent::LocalPeerId(peer_id) => {
            status.local_peer_id = Some(peer_id);
            (status, None)
        }
        NetworkEvent::Listening(address) => {
            status.listen_addr = Some(address);
            (status, None)
        }
        NetworkEvent::PeerDiscovered(peer_id) => {
            let inserted = status.peers.insert(peer_id.clone());
            let effect = inserted.then(|| {
                AppEffect::Log(format!(
                    "[network] peer discovered: {}",
                    short_peer_id(&peer_id)
                ))
            });
            (status, effect)
        }
        NetworkEvent::PeerExpired(peer_id) => {
            let removed = status.peers.remove(&peer_id);
            let effect = removed.then(|| {
                AppEffect::Log(format!(
                    "[network] peer expired: {}",
                    short_peer_id(&peer_id)
                ))
            });
            (status, effect)
        }
        NetworkEvent::Subscribed(topic) => (
            status,
            Some(AppEffect::Log(format!("[network] subscribed to {topic}"))),
        ),
        NetworkEvent::MessageReceived { topic, payload } => (
            status,
            Some(AppEffect::Log(format!(
                "[network] message on {topic}: {} byte(s)",
                payload.len()
            ))),
        ),
        NetworkEvent::Log(message) | NetworkEvent::Error(message) => {
            (status, Some(AppEffect::Log(format!("[network] {message}"))))
        }
    }
}

#[test]
fn diff_text_detects_single_character_insert() {
    let delta = diff_text("ab", "acb").expect("text changed");

    assert_eq!(delta.prefix_len, 1);
    assert_eq!(delta.removed_count, 0);
    assert_eq!(delta.inserted, vec!['c']);
}

#[test]
fn diff_text_detects_single_character_delete() {
    let delta = diff_text("acb", "ab").expect("text changed");

    assert_eq!(delta.prefix_len, 1);
    assert_eq!(delta.removed_count, 1);
    assert!(delta.inserted.is_empty());
}

#[test]
fn diff_text_detects_middle_replacement() {
    let delta = diff_text("abc", "axc").expect("text changed");

    assert_eq!(delta.prefix_len, 1);
    assert_eq!(delta.removed_count, 1);
    assert_eq!(delta.inserted, vec!['x']);
}

#[test]
fn deterministic_seed_allows_followup_insert_to_apply_on_other_replica() {
    let document_id = default_document_id();
    let initial_text = "Hello";

    let mut alice = TextCrdt::new(Uuid::from_u128(10));
    let mut bob = TextCrdt::new(Uuid::from_u128(20));

    seed_text_crdt(&mut alice, document_id, initial_text);
    seed_text_crdt(&mut bob, document_id, initial_text);

    let insert = alice.insert(initial_text.chars().count(), '!');
    bob.apply(insert);

    assert_eq!(alice.value(), "Hello!");
    assert_eq!(bob.value(), "Hello!");
}

#[test]
fn bootstrap_sync_replaces_local_seed_and_replays_buffered_remote_operations() {
    let document_id = default_document_id();

    let mut local = TextCrdt::new(Uuid::from_u128(10));
    seed_text_crdt(&mut local, document_id, "Old");

    let mut remote = TextCrdt::new(Uuid::from_u128(20));
    seed_text_crdt(&mut remote, document_id, "Hello");
    let authoritative_operations = remote.applied_operations().to_vec();
    let buffered_remote_operation = remote.insert(5, '!');

    let rebuilt = rebuild_text_crdt(
        Uuid::from_u128(30),
        &authoritative_operations,
        std::slice::from_ref(&buffered_remote_operation),
    );

    assert_eq!(local.value(), "Old");
    assert_eq!(rebuilt.value(), remote.value());
}
