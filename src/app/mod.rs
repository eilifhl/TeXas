pub mod build;
pub mod platform;
pub mod theme;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use eframe::egui::Context;
use tokio::runtime::Runtime;

use crate::network::{self, NetworkEvent, NetworkHandle};
use build::run_latexmk;
use platform::open_path_with_default_app;
use theme::{ThemeMode, configure_theme};

pub struct TexasApp {
    editor_text: String,
    output_text: String,
    last_pdf_path: Option<PathBuf>,
    theme_mode: ThemeMode,
    network: NetworkHandle,
    network_status: NetworkStatus,
    _runtime: Runtime,
}

#[derive(Default)]
struct NetworkStatus {
    local_peer_id: Option<String>,
    listen_addr: Option<String>,
    peer_count: usize,
}

impl TexasApp {
    pub fn new(ctx: &Context, runtime: Runtime) -> Result<Self> {
        let theme_mode = ThemeMode::Dark;
        configure_theme(ctx, theme_mode);

        let network = network::start(runtime.handle())?;

        Ok(Self {
            editor_text: SAMPLE_DOCUMENT.to_owned(),
            output_text: "No build output yet.".to_owned(),
            last_pdf_path: None,
            theme_mode,
            network,
            network_status: NetworkStatus::default(),
            _runtime: runtime,
        })
    }

    pub(crate) fn compile(&mut self) {
        let result = run_latexmk(&self.editor_text);
        self.output_text = result.output;
        self.last_pdf_path = result.pdf_path;
    }

    pub(crate) fn open_pdf(&mut self) {
        let Some(pdf_path) = &self.last_pdf_path else {
            self.output_text = "No compiled PDF available to open.".to_owned();
            return;
        };

        if let Err(error) = open_path_with_default_app(pdf_path) {
            self.output_text = format!("Failed to open PDF:\n{}\n{error}", pdf_path.display());
        }
    }

    pub(crate) fn toggle_theme(&mut self, ctx: &Context) {
        self.theme_mode = self.theme_mode.toggle();
        configure_theme(ctx, self.theme_mode);
    }

    pub(crate) fn sync_network_state(&mut self) {
        for event in self.network.poll_events() {
            match event {
                NetworkEvent::LocalPeerId(peer_id) => {
                    self.network_status.local_peer_id = Some(peer_id);
                }
                NetworkEvent::Listening(address) => {
                    self.network_status.listen_addr = Some(address);
                }
                NetworkEvent::PeerDiscovered(peer_id) => {
                    self.network_status.peer_count += 1;
                    append_output_line(
                        &mut self.output_text,
                        format!("[network] peer discovered: {}", short_peer_id(&peer_id)),
                    );
                }
                NetworkEvent::PeerExpired(peer_id) => {
                    self.network_status.peer_count =
                        self.network_status.peer_count.saturating_sub(1);
                    append_output_line(
                        &mut self.output_text,
                        format!("[network] peer expired: {}", short_peer_id(&peer_id)),
                    );
                }
                NetworkEvent::Log(message) | NetworkEvent::Error(message) => {
                    append_output_line(&mut self.output_text, format!("[network] {message}"));
                }
            }
        }
    }

    pub(crate) fn connection_label(&self) -> String {
        let peer = self
            .network_status
            .local_peer_id
            .as_deref()
            .map(short_peer_id)
            .unwrap_or("starting");

        if let Some(address) = &self.network_status.listen_addr {
            format!(
                "Peer {peer} | {} peer(s) | {address}",
                self.network_status.peer_count
            )
        } else {
            format!("Peer {peer} | {} peer(s)", self.network_status.peer_count)
        }
    }
}

impl eframe::App for TexasApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.sync_network_state();
        self.render(ctx);
    }
}

fn append_output_line(output_text: &mut String, line: String) {
    if output_text == "No build output yet." {
        *output_text = line;
    } else {
        if !output_text.ends_with('\n') {
            output_text.push('\n');
        }
        output_text.push_str(&line);
    }
}

fn short_peer_id(peer_id: &str) -> &str {
    peer_id.get(..12).unwrap_or(peer_id)
}

const SAMPLE_DOCUMENT: &str = r#"\documentclass{article}
\usepackage{amsmath}
\usepackage{graphicx}

\title{Barebones TeXas Draft}
\author{eilif tihi}
\date{\today}

\begin{document}
\maketitle

\section{Introduction}
This is a minimal editor shell for drafting LaTeX.

\section{Method}
In the future, you should be able to edit this text in real-time, with a peer-to-peer connection.

\end{document}
"#;
