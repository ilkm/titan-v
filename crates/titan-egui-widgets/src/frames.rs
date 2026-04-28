//! Cards, inset shells, and dialog frame surfaces.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Sense, Stroke, Visuals};

use crate::theme::{
    card_shadow, ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, FORM_LABEL_WIDTH, PANEL_SPACING,
};

fn outline_stroke_from_visuals(visuals: &Visuals) -> Stroke {
    Stroke::new(
        1.0,
        visuals
            .widgets
            .noninteractive
            .bg_stroke
            .color
            .linear_multiply(0.55),
    )
}

fn card_outline_stroke(ui: &egui::Ui) -> Stroke {
    outline_stroke_from_visuals(ui.visuals())
}

fn paint_section_title_accent(ui: &mut egui::Ui, accent: Color32) {
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
pub(crate) fn inset_editor_shell(
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

/// Opaque modal frame (solid white) for small dialogs.
pub fn opaque_dialog_frame(ui: &egui::Ui) -> Frame {
    Frame::NONE
        .fill(Color32::WHITE)
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(Margin::symmetric(24, 20))
        .shadow(card_shadow())
        .stroke(outline_stroke_from_visuals(ui.visuals()))
}

/// Same as [`opaque_dialog_frame`] when only [`egui::Context`] is available (e.g. top-level paint).
pub fn opaque_dialog_frame_ctx(ctx: &egui::Context) -> Frame {
    Frame::NONE
        .fill(Color32::WHITE)
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(Margin::symmetric(24, 20))
        .shadow(card_shadow())
        .stroke(outline_stroke_from_visuals(&ctx.style().visuals))
}
