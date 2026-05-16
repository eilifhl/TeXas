pub mod crdt;

use std::{
    env,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use eframe::egui::{
    self, Align, CentralPanel, Color32, Context, CornerRadius, FontFamily, FontId, Frame,
    Layout, Margin, RichText, ScrollArea, SidePanel, Stroke, TextEdit, TopBottomPanel, Vec2,
};
use uuid::Uuid;
use crate::crdt::text_crdt::TextCrdt;

const CARBON_BLUE_60: Color32 = Color32::from_rgb(15, 98, 254);
const CARBON_BLUE_80: Color32 = Color32::from_rgb(69, 137, 255);
const CARBON_GRAY_100: Color32 = Color32::from_rgb(22, 22, 22);
const CARBON_GRAY_80: Color32 = Color32::from_rgb(57, 57, 57);
const CARBON_GRAY_70: Color32 = Color32::from_rgb(82, 82, 82);
const CARBON_GRAY_60: Color32 = Color32::from_rgb(109, 109, 109);
const CARBON_GRAY_30: Color32 = Color32::from_rgb(198, 198, 198);
const CARBON_GRAY_10: Color32 = Color32::from_rgb(244, 244, 244);
const CARBON_GRAY_90: Color32 = Color32::from_rgb(38, 38, 38);
const CARBON_GRAY_20: Color32 = Color32::from_rgb(238, 238, 238);
const GOV_UK_BLACK: Color32 = Color32::from_rgb(11, 12, 12);
const GOV_UK_WHITE: Color32 = Color32::from_rgb(255, 255, 255);

fn main() -> eframe::Result<()> {
    let _text_crdt = TextCrdt::new(Uuid::new_v4());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("TeXas")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TeXas",
        options,
        Box::new(|cc| {
            configure_theme(&cc.egui_ctx, ThemeMode::Dark);
            Ok(Box::new(LatexEditorApp::default()))
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
}

fn configure_theme(ctx: &Context, theme_mode: ThemeMode) {
    let mut style = (*ctx.style()).clone();

    style.visuals = match theme_mode {
        ThemeMode::Light => egui::Visuals::light(),
        ThemeMode::Dark => egui::Visuals::dark(),
    };

    match theme_mode {
        ThemeMode::Light => {
            style.visuals.window_fill = CARBON_GRAY_10;
            style.visuals.panel_fill = CARBON_GRAY_10;
            style.visuals.override_text_color = Some(GOV_UK_BLACK);
            style.visuals.extreme_bg_color = GOV_UK_WHITE;
            style.visuals.code_bg_color = GOV_UK_WHITE;
            style.visuals.faint_bg_color = CARBON_GRAY_20;
            style.visuals.widgets.noninteractive.bg_fill = CARBON_GRAY_10;
            style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, GOV_UK_BLACK);
            style.visuals.widgets.inactive.bg_fill = GOV_UK_WHITE;
            style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, GOV_UK_BLACK);
            style.visuals.widgets.hovered.bg_fill = CARBON_GRAY_10;
            style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, GOV_UK_BLACK);
            style.visuals.widgets.active.bg_fill = CARBON_GRAY_20;
            style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, GOV_UK_BLACK);
            style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, CARBON_GRAY_30);
            style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, CARBON_BLUE_60);
            style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, CARBON_BLUE_60);
            style.visuals.window_stroke = Stroke::new(1.0, CARBON_GRAY_30);
            style.visuals.selection.bg_fill = Color32::from_rgb(210, 226, 255);
            style.visuals.selection.stroke = Stroke::new(1.0, CARBON_BLUE_60);
        }
        ThemeMode::Dark => {
            style.visuals.window_fill = CARBON_GRAY_100;
            style.visuals.panel_fill = CARBON_GRAY_100;
            style.visuals.override_text_color = Some(GOV_UK_WHITE);
            style.visuals.extreme_bg_color = CARBON_GRAY_90;
            style.visuals.code_bg_color = CARBON_GRAY_90;
            style.visuals.faint_bg_color = CARBON_GRAY_80;
            style.visuals.widgets.noninteractive.bg_fill = CARBON_GRAY_100;
            style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, CARBON_GRAY_10);
            style.visuals.widgets.inactive.bg_fill = CARBON_GRAY_90;
            style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, GOV_UK_WHITE);
            style.visuals.widgets.hovered.bg_fill = CARBON_GRAY_80;
            style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, GOV_UK_WHITE);
            style.visuals.widgets.active.bg_fill = CARBON_GRAY_70;
            style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, GOV_UK_WHITE);
            style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, CARBON_GRAY_30);
            style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, CARBON_BLUE_80);
            style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, CARBON_BLUE_80);
            style.visuals.window_stroke = Stroke::new(1.0, CARBON_GRAY_30);
            style.visuals.selection.bg_fill = CARBON_BLUE_60;
            style.visuals.selection.stroke = Stroke::new(1.0, CARBON_BLUE_80);
        }
    }

    style.visuals.window_corner_radius = CornerRadius::ZERO;
    style.visuals.menu_corner_radius = CornerRadius::ZERO;
    style.visuals.window_shadow = eframe::epaint::Shadow::NONE;
    style.visuals.popup_shadow = eframe::epaint::Shadow::NONE;
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.active.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.open.corner_radius = CornerRadius::ZERO;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 7.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.window_margin = Margin::same(12);

    style.text_styles = [
        (
            egui::TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            FontId::new(15.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            FontId::new(15.0, FontFamily::Monospace),
        ),
    ]
    .into();

    ctx.set_style(style);
}

struct LatexEditorApp {
    editor_text: String,
    build_output: String,
    last_pdf_path: Option<PathBuf>,
    theme_mode: ThemeMode,
}

impl Default for LatexEditorApp {
    fn default() -> Self {
        Self {
            editor_text: SAMPLE_DOCUMENT.to_owned(),
            build_output: "No build output yet.".to_owned(),
            last_pdf_path: None,
            theme_mode: ThemeMode::Dark,
        }
    }
}

impl LatexEditorApp {
    fn compile(&mut self) {
        let result = run_latexmk(&self.editor_text);
        self.build_output = result.output;
        self.last_pdf_path = result.pdf_path;
    }

    fn open_pdf(&mut self) {
        let Some(pdf_path) = &self.last_pdf_path else {
            self.build_output = "No compiled PDF available to open.".to_owned();
            return;
        };

        match open_path_with_default_app(pdf_path) {
            Ok(_) => {}
            Err(error) => {
                self.build_output = format!(
                    "Failed to open PDF:\n{}\n{error}",
                    pdf_path.display()
                );
            }
        }
    }

    fn toggle_theme(&mut self, ctx: &Context) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        configure_theme(ctx, self.theme_mode);
    }

}

impl eframe::App for LatexEditorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let (panel_fill, surface_fill, title_color, label_color, editor_text_color) =
            match self.theme_mode {
                ThemeMode::Light => (
                    GOV_UK_WHITE,
                    GOV_UK_WHITE,
                    GOV_UK_BLACK,
                    CARBON_GRAY_60,
                    GOV_UK_BLACK,
                ),
                ThemeMode::Dark => (
                    CARBON_GRAY_90,
                    CARBON_GRAY_100,
                    GOV_UK_WHITE,
                    CARBON_GRAY_10,
                    GOV_UK_WHITE,
                ),
            };

        TopBottomPanel::top("top_bar")
            .frame(
                Frame::new()
                    .fill(panel_fill)
                    .stroke(Stroke::new(1.0, CARBON_GRAY_30))
                    .inner_margin(Margin::symmetric(14, 12)),
            )
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("TeXas")
                                .size(22.0)
                                .strong()
                                .color(title_color),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [112.0, 36.0],
                                egui::Button::new(match self.theme_mode {
                                    ThemeMode::Light => "Dark mode",
                                    ThemeMode::Dark => "Light mode",
                                }),
                            )
                            .clicked()
                        {
                            self.toggle_theme(ctx);
                        }

                        let open_pdf = ui.add_enabled_ui(self.last_pdf_path.is_some(), |ui| {
                            ui.add_sized([112.0, 36.0], egui::Button::new("Open PDF"))
                        });
                        if open_pdf.inner.clicked() {
                            self.open_pdf();
                        }

                        if ui
                            .add_sized([112.0, 36.0], egui::Button::new("Compile"))
                            .clicked()
                        {
                            self.compile();
                        }
                    });
                });
            });

        SidePanel::right("meta_panel")
            .resizable(true)
            .default_width(290.0)
            .min_width(240.0)
            .frame(
                Frame::new()
                    .fill(panel_fill)
                    .stroke(Stroke::new(1.0, CARBON_GRAY_30))
                    .inner_margin(Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Build output")
                        .size(13.0)
                        .strong()
                        .color(label_color),
                );
                ui.add_space(4.0);

                let output_height = ui.available_height();
                Frame::new()
                    .fill(surface_fill)
                    .stroke(Stroke::new(1.0, CARBON_GRAY_30))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_min_height(output_height);
                        ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&self.build_output)
                                        .small()
                                        .monospace()
                                        .color(editor_text_color),
                                );
                            });
                    });
            });

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(surface_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("main.tex")
                        .size(16.0)
                        .strong()
                        .color(title_color),
                );
                ui.add_space(8.0);

                let editor_height = ui.available_height();
                Frame::new()
                    .fill(panel_fill)
                    .stroke(Stroke::new(1.0, CARBON_GRAY_30))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_min_height(editor_height);
                        TextEdit::multiline(&mut self.editor_text)
                            .desired_width(ui.available_width())
                            .desired_rows(30)
                            .min_size(ui.available_size())
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Start writing LaTeX here...")
                            .text_color(editor_text_color)
                            .show(ui);
                    });
            });
    }
}

struct CompileResult {
    output: String,
    pdf_path: Option<PathBuf>,
}

fn run_latexmk(editor_text: &str) -> CompileResult {
    let build_dir = match create_build_dir() {
        Ok(path) => path,
        Err(error) => {
            return CompileResult {
                output: format!("Failed to prepare build directory:\n{error}"),
                pdf_path: None,
            };
        }
    };

    let tex_path = build_dir.join("main.tex");
    if let Err(error) = fs::write(&tex_path, editor_text) {
        return CompileResult {
            output: format!("Failed to write {}:\n{error}", tex_path.display()),
            pdf_path: None,
        };
    }

    let output = match Command::new("latexmk")
        .args(["-pdf", "-interaction=nonstopmode", "-halt-on-error", "main.tex"])
        .current_dir(&build_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return CompileResult {
                output: format!(
                    "Failed to run latexmk in {}:\n{error}",
                    build_dir.display()
                ),
                pdf_path: None,
            };
        }
    };

    let mut text = String::new();
    text.push_str("$ latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex\n");
    text.push_str(&format!("Working directory: {}\n\n", build_dir.display()));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        text.push_str(&stdout);
        if !stdout.ends_with('\n') {
            text.push('\n');
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("[stderr]\n");
        text.push_str(&stderr);
        if !stderr.ends_with('\n') {
            text.push('\n');
        }
    }

    text.push_str(&format!(
        "\nExit status: {}\n",
        output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "terminated by signal".to_owned())
    ));

    let pdf_path = build_dir.join("main.pdf");
    if pdf_path.exists() {
        text.push_str(&format!("PDF: {}\n", pdf_path.display()));
        CompileResult {
            output: text,
            pdf_path: Some(pdf_path),
        }
    } else {
        CompileResult {
            output: text,
            pdf_path: None,
        }
    }
}

fn create_build_dir() -> std::io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let build_dir = env::temp_dir().join(format!("texas-{timestamp}"));
    fs::create_dir_all(&build_dir)?;
    Ok(build_dir)
}

fn discover_wayland_display() -> Option<String> {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")?;
    let mut candidates = fs::read_dir(runtime_dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("wayland-"))
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.into_iter().next()
}

fn open_path_with_default_app(path: &PathBuf) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let mut command = Command::new("xdg-open");
        command.arg(path);

        if env::var_os("WAYLAND_DISPLAY").is_none() {
            if let Some(wayland_display) = discover_wayland_display() {
                command.env("WAYLAND_DISPLAY", wayland_display);
            }
        }

        command.spawn().map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .map(|_| ())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "opening PDFs is not supported on this platform",
        ))
    }
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
