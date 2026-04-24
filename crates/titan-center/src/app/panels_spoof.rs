//! Host spoof profile apply / preview.

use egui::{Align2, RichText};

use super::constants::CONTENT_COLUMN_GAP;
use super::i18n::{t, Msg};
use super::widgets::{
    confirm_dialog_frame, form_field_row, inset_editor_shell, primary_button, section_card,
    subtle_button,
};
use super::CenterApp;

impl CenterApp {
    pub(super) fn panel_spoof_pipeline(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let inner = ui.available_width();
        let gap = CONTENT_COLUMN_GAP;
        let lang = self.ui_lang;
        if inner >= 560.0 {
            let half = ((inner - gap).max(0.0)) * 0.5;
            ui.horizontal(|ui| {
                ui.set_min_width(inner);
                ui.vertical(|ui| {
                    ui.set_width(half);
                    section_card(ui, t(lang, Msg::SpoofCardTitle), |ui| {
                        self.spoof_options_body(ui);
                    });
                });
                ui.add_space(gap);
                ui.vertical(|ui| {
                    ui.set_width(half);
                    section_card(ui, t(lang, Msg::CardActions), |ui| {
                        self.spoof_actions_body(ui);
                    });
                });
            });
        } else {
            section_card(ui, t(lang, Msg::SpoofCardTitle), |ui| {
                self.spoof_options_body(ui);
                ui.add_space(10.0);
                self.spoof_actions_body(ui);
            });
        }

        if self.pending_spoof_confirm_apply {
            self.show_spoof_confirm_window(ctx, ui);
        }
    }

    fn spoof_options_body(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(t(self.ui_lang, Msg::SpoofBlurb))
                .small()
                .color(ui.visuals().widgets.inactive.text_color()),
        );
        ui.add_space(6.0);
        form_field_row(
            ui,
            RichText::new(t(self.ui_lang, Msg::TargetVmLabel)).small(),
            |ui| {
                inset_editor_shell(ui, ui.spacing().interact_size.y.max(30.0), |ui| {
                    ui.add_enabled_ui(!self.net_busy, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.spoof_target_vm)
                                .desired_width(ui.available_width())
                                .hint_text(t(self.ui_lang, Msg::HintTargetVm)),
                        );
                    });
                });
            },
        );
        ui.checkbox(
            &mut self.spoof_dynamic_mac,
            t(self.ui_lang, Msg::ChkDynamicMac),
        );
        ui.checkbox(
            &mut self.spoof_disable_checkpoints,
            t(self.ui_lang, Msg::ChkDisableCkpt),
        );
    }

    fn spoof_actions_body(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let can = !self.net_busy && self.host_connected;
            if subtle_button(ui, t(self.ui_lang, Msg::BtnPreviewDryRun), can).clicked() {
                self.spawn_spoof_apply(true);
            }
            if subtle_button(
                ui,
                t(self.ui_lang, Msg::BtnApplyEllipsis),
                can && !self.pending_spoof_confirm_apply,
            )
            .clicked()
            {
                self.pending_spoof_confirm_apply = true;
            }
        });
    }

    fn show_spoof_confirm_window(&mut self, ctx: &egui::Context, outer_ui: &egui::Ui) {
        egui::Window::new(t(self.ui_lang, Msg::WinConfirmSpoofTitle))
            .frame(confirm_dialog_frame(outer_ui))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::WinConfirmSpoofBody))
                        .line_height(Some(20.0)),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if subtle_button(ui, t(self.ui_lang, Msg::BtnCancel), true).clicked() {
                        self.pending_spoof_confirm_apply = false;
                        self.last_action = super::i18n::log_spoof_cancelled(self.ui_lang);
                    }
                    if primary_button(ui, t(self.ui_lang, Msg::BtnConfirmApply), true).clicked() {
                        self.pending_spoof_confirm_apply = false;
                        self.spawn_spoof_apply(false);
                        self.last_action = super::i18n::log_spoof_dispatched(self.ui_lang);
                    }
                });
            });
    }
}
