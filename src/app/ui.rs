use eframe::egui::{
    self, Align, CentralPanel, Context, Frame, Layout, Margin, RichText, ScrollArea, SidePanel,
    Stroke, TextEdit, TopBottomPanel,
};

use super::{TexasApp, theme::ThemeMode, theme::palette};

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
                            .add_sized([112.0, 36.0], egui::Button::new("Send test"))
                            .clicked()
                        {
                            self.publish_test_message();
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

                let output_height = ui.available_height();
                Frame::new()
                    .fill(colors.surface_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_min_height(output_height);
                        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
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
                ui.add_space(8.0);

                let editor_height = ui.available_height();
                Frame::new()
                    .fill(colors.panel_fill)
                    .stroke(Stroke::new(1.0, colors.border_color))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_min_height(editor_height);
                        TextEdit::multiline(&mut self.model.editor_text)
                            .desired_width(ui.available_width())
                            .desired_rows(30)
                            .min_size(ui.available_size())
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Start writing LaTeX here...")
                            .text_color(colors.text_color)
                            .show(ui);
                    });
            });
    }
}
