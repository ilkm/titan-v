//! Reusable egui frames and buttons.

use egui::text::CursorRange;
use egui::widgets::text_edit::TextEditOutput;
use egui::widgets::Button;
use egui::{pos2, Color32, CornerRadius, Frame, Margin, Response, RichText, Sense, Stroke};

use super::constants::{
    card_shadow, ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, FORM_LABEL_WIDTH, PANEL_SPACING,
};

fn card_outline_stroke(ui: &egui::Ui) -> Stroke {
    Stroke::new(
        1.0,
        ui.visuals()
            .widgets
            .noninteractive
            .bg_stroke
            .color
            .linear_multiply(0.55),
    )
}

fn paint_section_title_accent(ui: &mut egui::Ui, accent: egui::Color32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 15.0), Sense::empty());
    ui.painter().rect_filled(r, CornerRadius::same(3), accent);
}

/// Left-aligned label column + field; labels share one left edge (wraps inside fixed width).
pub fn form_field_row(ui: &mut egui::Ui, label: RichText, add_field: impl FnOnce(&mut egui::Ui)) {
    let label_w = (ui.available_width() * 0.32).clamp(72.0, FORM_LABEL_WIDTH);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.vertical(|ui| {
            ui.set_width(label_w);
            ui.label(label);
        });
        ui.add_space(10.0);
        add_field(ui);
    });
    ui.add_space(8.0);
}

pub fn section_card(ui: &mut egui::Ui, title: &str, add_body: impl FnOnce(&mut egui::Ui)) {
    Frame::NONE
        .fill(CARD_SURFACE)
        .corner_radius(CARD_CORNER_RADIUS)
        .stroke(card_outline_stroke(ui))
        .shadow(card_shadow())
        .outer_margin(Margin::symmetric(0, 3))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    paint_section_title_accent(ui, ACCENT);
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .size(14.0)
                            .extra_letter_spacing(0.2)
                            .color(ACCENT),
                    );
                });
                ui.add_space(8.0);
                add_body(ui);
            });
        });
    ui.add_space(PANEL_SPACING + 2.0);
}

/// Inset field surface (same depth as window preview placeholder) for text inputs.
pub fn inset_editor_shell(
    ui: &mut egui::Ui,
    min_height: f32,
    add_body: impl FnOnce(&mut egui::Ui),
) {
    let fill = ui
        .visuals()
        .widgets
        .noninteractive
        .bg_fill
        .linear_multiply(1.08);
    let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    Frame::NONE
        .fill(fill)
        .corner_radius(CornerRadius::same(8))
        .stroke(stroke)
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_height(min_height.max(28.0));
            add_body(ui);
        });
    ui.add_space(6.0);
}

pub fn primary_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let fill = if enabled {
        ACCENT
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let label = if enabled {
        RichText::new(text).strong().color(egui::Color32::WHITE)
    } else {
        RichText::new(text).strong()
    };
    ui.add_enabled(
        enabled,
        Button::new(label)
            .fill(fill)
            .corner_radius(CornerRadius::same(8)),
    )
}

pub fn subtle_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        Button::new(text).corner_radius(CornerRadius::same(8)),
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
    ui.add_enabled(
        enabled,
        Button::new(label)
            .fill(fill)
            .min_size(DLG_BTN_MIN)
            .corner_radius(DLG_BTN_RADIUS),
    )
}

/// Secondary action for compact modals (matches [`primary_button_large`] height).
pub fn subtle_button_large(ui: &mut egui::Ui, text: &str, enabled: bool) -> Response {
    let label = RichText::new(text).strong().size(DLG_BTN_FONT);
    ui.add_enabled(
        enabled,
        Button::new(label)
            .min_size(DLG_BTN_MIN)
            .corner_radius(DLG_BTN_RADIUS),
    )
}

pub fn confirm_dialog_frame(ui: &egui::Ui) -> Frame {
    Frame::NONE
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(Margin::same(22))
        .shadow(card_shadow())
        .stroke(card_outline_stroke(ui))
}

/// Opaque modal frame (solid white) for small dialogs.
#[must_use]
pub fn opaque_dialog_frame(ui: &egui::Ui) -> Frame {
    Frame::NONE
        .fill(Color32::WHITE)
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(Margin::symmetric(24, 20))
        .shadow(card_shadow())
        .stroke(card_outline_stroke(ui))
}

const UNDERLINE_IDLE: Color32 = Color32::from_rgb(148, 163, 184);

/// Borderless single-line edit with only a bottom rule; focused rule uses [`ACCENT`].
/// On focus gain, the caret jumps to the end of the text (same frame via persisted state).
///
/// `gap_below_underline`: extra vertical space after the rule (e.g. `8.0` between dialog fields;
/// use `0.0` for inline rows where layout height must not grow on focus).
pub fn dialog_underline_text_row_gap(
    ui: &mut egui::Ui,
    add_textedit: impl FnOnce(&mut egui::Ui) -> TextEditOutput,
    gap_below_underline: f32,
) -> Response {
    let output = add_textedit(ui);
    let r = output.response;
    if r.gained_focus() {
        let cursor_end = CursorRange::one(output.galley.end());
        let mut state = output.state;
        state.cursor.set_range(Some(cursor_end));
        state.store(ui.ctx(), r.id);
    }
    let y = r.rect.bottom() + 1.0;
    let color = if r.has_focus() {
        ACCENT
    } else {
        UNDERLINE_IDLE
    };
    // Compact rows (`gap_below_underline == 0`): keep stroke width constant so focus does not nudge layout.
    let stroke_w = if gap_below_underline > 0.0 {
        if r.has_focus() {
            1.5
        } else {
            1.0
        }
    } else {
        1.0
    };
    ui.painter().line_segment(
        [pos2(r.rect.min.x, y), pos2(r.rect.max.x, y)],
        Stroke::new(stroke_w, color),
    );
    if gap_below_underline > 0.0 {
        ui.add_space(gap_below_underline);
    }
    r
}

pub fn dialog_underline_text_row(
    ui: &mut egui::Ui,
    add_textedit: impl FnOnce(&mut egui::Ui) -> TextEditOutput,
) -> Response {
    dialog_underline_text_row_gap(ui, add_textedit, 8.0)
}
