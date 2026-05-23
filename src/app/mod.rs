pub mod build;
pub mod platform;
pub mod theme;
mod ui;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use eframe::egui::Context;
use tokio::runtime::Runtime;

use crate::network::{self, DEFAULT_DOCUMENT_TOPIC, NetworkEvent, NetworkHandle};
use build::run_latexmk;
use platform::open_path_with_default_app;
use theme::{ThemeMode, configure_theme};

pub struct TexasApp {
    model: AppModel,
    network: NetworkHandle,
    _runtime: Runtime,
}

struct AppModel {
    editor_text: String,
    document_path: PathBuf,
    output_text: String,
    last_pdf_path: Option<PathBuf>,
    theme_mode: ThemeMode,
    network_status: NetworkStatus,
}

#[derive(Clone, Default)]
struct NetworkStatus {
    local_peer_id: Option<String>,
    listen_addr: Option<String>,
    peers: HashSet<String>,
}

enum AppEffect {
    Log(String),
}

impl AppModel {
    fn new(theme_mode: ThemeMode) -> Result<Self> {
        let document_path = default_document_path();
        let editor_text = load_document(&document_path)?;

        Ok(Self {
            editor_text,
            document_path,
            output_text: "No build output yet.".to_owned(),
            last_pdf_path: None,
            theme_mode,
            network_status: NetworkStatus::default(),
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

        Ok(Self {
            model: AppModel::new(theme_mode)?,
            network,
            _runtime: runtime,
        })
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

    pub(crate) fn publish_test_message(&mut self) {
        let message = format!("test message from {}", self.connection_label());
        match self
            .network
            .publish(DEFAULT_DOCUMENT_TOPIC, message.clone().into_bytes())
        {
            Ok(()) => {
                self.model.apply_effect(AppEffect::Log(format!(
                    "[network] queued test message: {message}"
                )));
            }
            Err(error) => {
                self.model.apply_effect(AppEffect::Log(format!(
                    "[network] failed to queue test message: {error}"
                )));
            }
        }
    }

    pub(crate) fn toggle_theme(&mut self, ctx: &Context) {
        self.model.theme_mode = self.model.theme_mode.toggle();
        configure_theme(ctx, self.model.theme_mode);
    }

    pub(crate) fn sync_network_state(&mut self) {
        for event in self.network.poll_events() {
            if let Some(effect) = self.model.apply_network_event(event) {
                self.model.apply_effect(effect);
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
}

impl eframe::App for TexasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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

fn default_document_path() -> PathBuf {
    PathBuf::from("main.tex")
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
