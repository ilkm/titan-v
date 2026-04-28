//! Standard action buttons (primary / subtle / modal sizes / overlay).

use egui::widgets::Button;
use egui::{Color32, CornerRadius, Rect, Response, RichText, Stroke};

use crate::theme::{ACCENT, ACCENT_DIM};

/// Modal / form secondary button outline (slate-300); pairs with theme `weak_bg_fill` when fill is unset.
const BTN_SUBTLE_STROKE: Color32 = Color32::from_rgb(203, 213, 225);

fn primary_btn_outline(ui: &egui::Ui, enabled: bool) -> Stroke {
    Stroke::new(
        1.0,
        if enabled {
            ACCENT_DIM
        } else {
            ui.visuals().widgets.inactive.bg_stroke.color
        },
    )
}

pub fn subtle_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        Button::new(RichText::new(text).strong())
            .stroke(Stroke::new(1.0, BTN_SUBTLE_STROKE))
            .corner_radius(CornerRadius::same(8)),
    )
}

const DLG_BTN_MIN: egui::Vec2 = egui::vec2(120.0, 44.0);
const DLG_BTN_FONT: f32 = 15.0;
const DLG_BTN_RADIUS: CornerRadius = CornerRadius::same(10);

/// Primary action for compact modals (taller hit target, larger label).
pub fn primary_button_large(ui: &mut egui::Ui, text: &str, enabled: bool) -> Response {
    let fill = if enabled {
        ACCENT
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let label = if enabled {
        RichText::new(text)
            .strong()
            .size(DLG_BTN_FONT)
            .color(Color32::WHITE)
    } else {
        RichText::new(text).strong().size(DLG_BTN_FONT)
    };
    let stroke = primary_btn_outline(ui, enabled);
    ui.add_enabled(
        enabled,
        Button::new(label)
            .fill(fill)
            .stroke(stroke)
            .min_size(DLG_BTN_MIN)
            .corner_radius(DLG_BTN_RADIUS),
    )
}

/// Secondary action for compact modals (matches [`primary_button_large`] height).
///
/// Uses a fixed outline so the control reads on white dialogs; fill follows theme interaction
/// (hover/active) because [`Button::fill`] is not set.
pub fn subtle_button_large(ui: &mut egui::Ui, text: &str, enabled: bool) -> Response {
    let label = RichText::new(text).strong().size(DLG_BTN_FONT);
    ui.add_enabled(
        enabled,
        Button::new(label)
            .stroke(Stroke::new(1.0, BTN_SUBTLE_STROKE))
            .min_size(DLG_BTN_MIN)
            .corner_radius(DLG_BTN_RADIUS),
    )
}

/// Destructive action on desktop preview overlay (fixed rect, dark glass + red label).
pub fn danger_preview_delete_button(ui: &mut egui::Ui, rect: Rect, label: &str) -> Response {
    let red = Color32::from_rgb(255, 72, 72);
    let btn = Button::new(RichText::new(label).color(red))
        .fill(Color32::from_black_alpha(45))
        .stroke(Stroke::new(1.0, Color32::from_rgb(200, 55, 55)));
    ui.put(rect, btn)
}

/// Secondary action on desktop preview overlay (fixed rect, light glass + white label).
pub fn preview_overlay_configure_button(ui: &mut egui::Ui, rect: Rect, label: &str) -> Response {
    let btn = Button::new(RichText::new(label).color(Color32::WHITE))
        .fill(Color32::from_white_alpha(36))
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(90)));
    ui.put(rect, btn)
}
