//! Light visuals and typography aligned with Titan Center.

use egui::{FontId, Margin, Stroke, TextStyle, Vec2};

use titan_egui_widgets::theme::{ACCENT, ACCENT_DIM, CARD_SURFACE};

use crate::host_app::constants::ERR_ROSE;

fn apply_light_visuals(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.dark_mode = false;
    let page = egui::Color32::from_rgb(232, 236, 244);
    v.window_fill = page;
    v.panel_fill = page;
    v.extreme_bg_color = egui::Color32::from_rgb(220, 228, 242);
    v.faint_bg_color = CARD_SURFACE;
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, egui::Color32::from_rgb(218, 220, 224));
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(241, 245, 249);
    v.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(203, 213, 225);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(224, 242, 254);
    v.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(147, 197, 253);
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(219, 234, 254);
    v.selection.bg_fill = egui::Color32::from_rgb(191, 219, 254);
    v.selection.stroke = Stroke::new(1.0, ACCENT_DIM);
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = egui::Color32::from_rgb(180, 120, 20);
    v.error_fg_color = ERR_ROSE;
    ctx.set_visuals(v);
}

fn apply_text_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 10.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.menu_margin = Margin::same(10);
    style.spacing.scroll = egui::style::ScrollStyle {
        floating: false,
        ..style.spacing.scroll
    };
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}

/// Page chrome (colors + spacing); call after [`crate::host_font::install_cjk_fonts`].
pub fn apply_host_chrome_theme(ctx: &egui::Context) {
    apply_light_visuals(ctx);
    apply_text_styles(ctx);
}
