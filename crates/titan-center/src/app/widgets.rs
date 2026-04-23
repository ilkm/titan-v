//! Reusable egui frames and buttons.

use egui::widgets::Button;
use egui::{CornerRadius, Frame, Layout, Margin, RichText, Sense, Shadow, Stroke};

use super::constants::{
    card_shadow, ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, DANGER_CARD_FILL, DANGER_CARD_STROKE,
    ERR_ROSE, FORM_LABEL_WIDTH, PANEL_SPACING,
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

/// Right-aligned label + field; keeps form columns visually aligned.
pub fn form_field_row(ui: &mut egui::Ui, label: RichText, add_field: impl FnOnce(&mut egui::Ui)) {
    let h = ui.spacing().interact_size.y.max(30.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.allocate_ui_with_layout(
            egui::vec2(FORM_LABEL_WIDTH, h),
            Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label(label);
            },
        );
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

pub fn section_card_danger(ui: &mut egui::Ui, title: &str, add_body: impl FnOnce(&mut egui::Ui)) {
    let stroke = Stroke::new(0.75, DANGER_CARD_STROKE.linear_multiply(0.88));
    let soft_shadow = Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: egui::Color32::from_rgba_unmultiplied(185, 28, 28, 10),
    };
    Frame::NONE
        .fill(DANGER_CARD_FILL)
        .corner_radius(CARD_CORNER_RADIUS)
        .stroke(stroke)
        .shadow(soft_shadow)
        .outer_margin(Margin::symmetric(0, 3))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    paint_section_title_accent(ui, ERR_ROSE);
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .size(14.0)
                            .extra_letter_spacing(0.2)
                            .color(ERR_ROSE),
                    );
                });
                ui.add_space(8.0);
                add_body(ui);
            });
        });
    ui.add_space(PANEL_SPACING + 2.0);
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

pub fn confirm_dialog_frame(ui: &egui::Ui) -> Frame {
    Frame::NONE
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(Margin::same(22))
        .shadow(card_shadow())
        .stroke(card_outline_stroke(ui))
}
