use eframe::egui::{
    self, Color32, Context, CornerRadius, FontFamily, FontId, Margin, Stroke, Vec2,
};

const CARBON_BLUE_60: Color32 = Color32::from_rgb(15, 98, 254);
const CARBON_BLUE_80: Color32 = Color32::from_rgb(69, 137, 255);
const CARBON_GRAY_100: Color32 = Color32::from_rgb(22, 22, 22);
const CARBON_GRAY_90: Color32 = Color32::from_rgb(38, 38, 38);
const CARBON_GRAY_80: Color32 = Color32::from_rgb(57, 57, 57);
const CARBON_GRAY_70: Color32 = Color32::from_rgb(82, 82, 82);
const CARBON_GRAY_60: Color32 = Color32::from_rgb(109, 109, 109);
const CARBON_GRAY_30: Color32 = Color32::from_rgb(198, 198, 198);
const CARBON_GRAY_20: Color32 = Color32::from_rgb(238, 238, 238);
const CARBON_GRAY_10: Color32 = Color32::from_rgb(244, 244, 244);
const GOV_UK_BLACK: Color32 = Color32::from_rgb(11, 12, 12);
const GOV_UK_WHITE: Color32 = Color32::from_rgb(255, 255, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub panel_fill: Color32,
    pub surface_fill: Color32,
    pub title_color: Color32,
    pub label_color: Color32,
    pub text_color: Color32,
    pub border_color: Color32,
}

pub fn palette(theme_mode: ThemeMode) -> Palette {
    match theme_mode {
        ThemeMode::Light => Palette {
            panel_fill: GOV_UK_WHITE,
            surface_fill: GOV_UK_WHITE,
            title_color: GOV_UK_BLACK,
            label_color: CARBON_GRAY_60,
            text_color: GOV_UK_BLACK,
            border_color: CARBON_GRAY_30,
        },
        ThemeMode::Dark => Palette {
            panel_fill: CARBON_GRAY_90,
            surface_fill: CARBON_GRAY_100,
            title_color: GOV_UK_WHITE,
            label_color: CARBON_GRAY_10,
            text_color: GOV_UK_WHITE,
            border_color: CARBON_GRAY_30,
        },
    }
}

pub fn configure_theme(ctx: &Context, theme_mode: ThemeMode) {
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
