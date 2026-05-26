use eframe::egui::{
    self, Align, CentralPanel, Context, Frame, Layout, Margin, RichText, ScrollArea, SidePanel,
    Stroke, TextEdit, TopBottomPanel,
};

use super::{theme::palette, theme::ThemeMode, TexasApp};

impl TexasApp {
    pub(super) fn render(&mut self, ctx: &Context) {
        let colors = palette(self.model.theme_mode);

        TopBottomPanel::top("top_bar")
            .frame(
                Frame::new()
                    .fill(colors.panel_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::symmetric(14, 12)),
            )
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("TeXas")
                                .size(22.0)
                                .strong()
                                .color(colors.title_color),
                        );
                        ui.label(
                            RichText::new(self.connection_label())
                                .small()
                                .color(colors.label_color),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [112.0, 36.0],
                                egui::Button::new(match self.model.theme_mode {
                                    ThemeMode::Light => "Dark mode",
                                    ThemeMode::Dark => "Light mode",
                                }),
                            )
                            .clicked()
                        {
                            self.toggle_theme(ctx);
                        }

                        let open_pdf = ui
                            .add_enabled_ui(self.model.last_pdf_path.is_some(), |ui| {
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

        SidePanel::right("build_output")
            .resizable(true)
            .default_width(320.0)
            .min_width(260.0)
            .frame(
                Frame::new()
                    .fill(colors.panel_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Build output")
                        .size(13.0)
                        .strong()
                        .color(colors.label_color),
                );
                ui.add_space(4.0);

                let output_frame = Frame::new()
                    .fill(colors.surface_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::same(10));
                let output_size =
                    (ui.available_size() - output_frame.total_margin().sum()).max(egui::Vec2::ZERO);

                output_frame.show(ui, |ui| {
                    ui.set_min_size(output_size);
                    ScrollArea::vertical()
                        .id_salt("build_output_scroll")
                        .stick_to_bottom(true)
                        .max_height(output_size.y)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new(&self.model.output_text)
                                    .small()
                                    .monospace()
                                    .color(colors.text_color),
                            );
                        });
                });
            });

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(colors.surface_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("main.tex")
                        .size(16.0)
                        .strong()
                        .color(colors.title_color),
                );
                if self.editor_locked() {
                    ui.label(
                        RichText::new("Waiting briefly for session sync before enabling editing.")
                            .small()
                            .color(colors.label_color),
                    );
                }
                ui.add_space(8.0);
                let editor_locked = self.editor_locked();

                let editor_frame = Frame::new()
                    .fill(colors.panel_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::same(12));
                let editor_size =
                    (ui.available_size() - editor_frame.total_margin().sum()).max(egui::Vec2::ZERO);

                editor_frame.show(ui, |ui| {
                    ui.set_min_size(editor_size);
                    ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .max_height(editor_size.y)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let previous_text = self.model.editor_text.clone();
                            let response = ui.add_sized(
                                [ui.available_width(), editor_size.y],
                                TextEdit::multiline(&mut self.model.editor_text)
                                    .desired_width(ui.available_width())
                                    .desired_rows(30)
                                    .font(egui::TextStyle::Monospace)
                                    .frame(false)
                                    .interactive(!editor_locked)
                                    .hint_text("Start writing LaTeX here...")
                                    .text_color(colors.text_color),
                            );

                            if response.changed() {
                                let next_text = self.model.editor_text.clone();
                                self.apply_local_editor_change(&previous_text, &next_text);
                            }
                        });
                });
            });
    }
}
