//! Bulk power with confirmation dialogs.

use egui::widgets::Button;
use egui::{Align2, CornerRadius, RichText};

use super::constants::{CONTENT_COLUMN_GAP, ERR_ROSE};
use super::i18n::{t, Msg, UiLang};
use super::widgets::{
    confirm_dialog_frame, inset_editor_shell, primary_button, section_card, subtle_button,
};
use super::CenterApp;

impl CenterApp {
    pub(super) fn panel_danger(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let inner = ui.available_width();
        let gap = CONTENT_COLUMN_GAP;
        let lang = self.ui_lang;
        if inner >= 560.0 {
            let half = ((inner - gap).max(0.0)) * 0.5;
            self.paint_danger_two_column(ui, inner, gap, half, lang);
        } else {
            self.paint_danger_stacked(ui, lang);
        }

        if self.pending_confirm_stop {
            self.window_confirm_stop(ctx, ui);
        }
        if self.pending_confirm_start {
            self.window_confirm_start(ctx, ui);
        }
    }

    fn paint_danger_two_column(
        &mut self,
        ui: &mut egui::Ui,
        inner: f32,
        gap: f32,
        half: f32,
        lang: UiLang,
    ) {
        ui.horizontal(|ui| {
            ui.set_min_width(inner);
            ui.vertical(|ui| {
                ui.set_width(half);
                section_card(ui, t(lang, Msg::DangerCardTitle), |ui| {
                    self.danger_vm_list_body(ui);
                });
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(half);
                section_card(ui, t(lang, Msg::CardActions), |ui| {
                    self.danger_actions_body(ui);
                });
            });
        });
    }

    fn paint_danger_stacked(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        section_card(ui, t(lang, Msg::DangerCardTitle), |ui| {
            self.danger_vm_list_body(ui);
            ui.add_space(10.0);
            self.danger_actions_body(ui);
        });
    }

    fn danger_vm_list_body(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(t(self.ui_lang, Msg::DangerBlurb))
                .small()
                .color(ui.visuals().widgets.inactive.text_color()),
        );
        ui.add_space(6.0);
        inset_editor_shell(ui, 72.0, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.bulk_vm_names)
                    .desired_width(f32::INFINITY)
                    .desired_rows(2)
                    .hint_text(t(self.ui_lang, Msg::HintBulkVms)),
            );
        });
    }

    fn danger_bulk_stop_clicked(&mut self, ui: &mut egui::Ui, idle: bool) -> bool {
        let stop_fill = if idle {
            ERR_ROSE.linear_multiply(0.85)
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        let stop_label = if idle {
            RichText::new(t(self.ui_lang, Msg::BtnBulkStop))
                .strong()
                .color(egui::Color32::WHITE)
        } else {
            RichText::new(t(self.ui_lang, Msg::BtnBulkStop)).strong()
        };
        ui.add_enabled(
            idle,
            Button::new(stop_label)
                .fill(stop_fill)
                .corner_radius(CornerRadius::same(8)),
        )
        .clicked()
    }

    fn danger_actions_body(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        let idle = !self.pending_confirm_stop && !self.pending_confirm_start;
        ui.horizontal_wrapped(|ui| {
            if primary_button(ui, t(self.ui_lang, Msg::BtnBulkStart), idle).clicked() {
                self.pending_confirm_start = true;
            }
            if self.danger_bulk_stop_clicked(ui, idle) {
                self.pending_confirm_stop = true;
            }
        });
    }

    fn danger_confirm_stop_row(&mut self, ui: &mut egui::Ui) {
        if subtle_button(ui, t(self.ui_lang, Msg::BtnCancel), true).clicked() {
            self.pending_confirm_stop = false;
            self.last_action = super::i18n::log_bulk_stop_cancelled(self.ui_lang);
        }
        if ui
            .add_enabled(
                true,
                Button::new(
                    RichText::new(t(self.ui_lang, Msg::BtnConfirmStop))
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .fill(ERR_ROSE.linear_multiply(0.9))
                .corner_radius(CornerRadius::same(8)),
            )
            .clicked()
        {
            self.pending_confirm_stop = false;
            self.dispatch_bulk_stop();
        }
    }

    fn window_confirm_stop(&mut self, ctx: &egui::Context, outer_ui: &egui::Ui) {
        egui::Window::new(t(self.ui_lang, Msg::WinConfirmStopTitle))
            .frame(confirm_dialog_frame(outer_ui))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(RichText::new(t(self.ui_lang, Msg::WinConfirmStopBody)).strong());
                ui.add_space(12.0);
                ui.horizontal(|ui| self.danger_confirm_stop_row(ui));
            });
    }

    fn window_confirm_start(&mut self, ctx: &egui::Context, outer_ui: &egui::Ui) {
        egui::Window::new(t(self.ui_lang, Msg::WinConfirmStartTitle))
            .frame(confirm_dialog_frame(outer_ui))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(RichText::new(t(self.ui_lang, Msg::WinConfirmStartBody)).strong());
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if subtle_button(ui, t(self.ui_lang, Msg::BtnCancel), true).clicked() {
                        self.pending_confirm_start = false;
                        self.last_action = super::i18n::log_bulk_start_cancelled(self.ui_lang);
                    }
                    if primary_button(ui, t(self.ui_lang, Msg::BtnConfirmStart), true).clicked() {
                        self.pending_confirm_start = false;
                        self.dispatch_bulk_start();
                    }
                });
            });
    }

    fn dispatch_bulk_stop(&mut self) {
        let names = self.parse_bulk_vm_names();
        if names.is_empty() {
            self.last_action = super::i18n::log_no_vm_names(self.ui_lang);
        } else {
            self.spawn_stop_vm_group(names);
            self.last_action = super::i18n::log_stop_dispatched(self.ui_lang);
        }
    }

    fn dispatch_bulk_start(&mut self) {
        let names = self.parse_bulk_vm_names();
        if names.is_empty() {
            self.last_action = super::i18n::log_no_vm_names(self.ui_lang);
        } else {
            self.spawn_start_vm_group(names);
            self.last_action = super::i18n::log_start_dispatched(self.ui_lang);
        }
    }

    fn parse_bulk_vm_names(&self) -> Vec<String> {
        self.bulk_vm_names
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
