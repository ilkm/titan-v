//! Standard action buttons (primary / subtle / modal sizes / overlay).

use egui::widgets::Button;
use egui::{Color32, CornerRadius, Rect, Response, RichText, Stroke, Vec2};

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

/// Inset for buttons that share the 4×2 padding rule (`x` = left/right, `y` = top/bottom; see egui `button_padding`).
const BTN_INSET_PADDING: Vec2 = Vec2::new(4.0, 2.0);
const DLG_BTN_MIN: Vec2 = Vec2::new(120.0, 40.0);
const DLG_BTN_FONT: f32 = 15.0;
const DLG_BTN_RADIUS: CornerRadius = CornerRadius::same(10);

fn with_inset_button_padding<T>(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui) -> T) -> T {
    let prev = ui.spacing().button_padding;
    ui.spacing_mut().button_padding = BTN_INSET_PADDING;
    let out = f(ui);
    ui.spacing_mut().button_padding = prev;
    out
}

/// Toolbar subtle button: same stroke/corner as [`subtle_button`], fixed inset padding, intrinsic width
/// (min height bumped when `button_padding.y` exceeds egui default so the extra inset is visible).
pub fn subtle_button_toolbar(ui: &mut egui::Ui, text: &str, enabled: bool) -> Response {
    // egui default `button_padding.y` is 1; we use 2 (+1px/side). After layout, height often equals
    // `interact_size.y` anyway, so the extra inset is invisible unless we raise the min height by 2px.
    const DEFAULT_BTN_PAD_Y: f32 = 1.0;
    let pad_delta_y = (BTN_INSET_PADDING.y - DEFAULT_BTN_PAD_Y).max(0.0) * 2.0;
    let min_h = ui.style().spacing.interact_size.y + pad_delta_y;
    with_inset_button_padding(ui, |ui| {
        ui.add_enabled(
            enabled,
            Button::new(RichText::new(text).strong())
                .stroke(Stroke::new(1.0, BTN_SUBTLE_STROKE))
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(0.0, min_h)),
        )
    })
}

/// Primary action for modals and form footers (min size + label per theme constants).
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
    with_inset_button_padding(ui, |ui| {
        ui.add_enabled(
            enabled,
            Button::new(label)
                .fill(fill)
                .stroke(stroke)
                .min_size(DLG_BTN_MIN)
                .corner_radius(DLG_BTN_RADIUS),
        )
    })
}

/// Secondary action for compact modals (matches [`primary_button_large`] height).
///
/// Uses a fixed outline so the control reads on white dialogs; fill follows theme interaction
/// (hover/active) because [`Button::fill`] is not set.
pub fn subtle_button_large(ui: &mut egui::Ui, text: &str, enabled: bool) -> Response {
    let label = RichText::new(text).strong().size(DLG_BTN_FONT);
    with_inset_button_padding(ui, |ui| {
        ui.add_enabled(
            enabled,
            Button::new(label)
                .stroke(Stroke::new(1.0, BTN_SUBTLE_STROKE))
                .min_size(DLG_BTN_MIN)
                .corner_radius(DLG_BTN_RADIUS),
        )
    })
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
