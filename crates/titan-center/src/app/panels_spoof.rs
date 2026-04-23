//! Host spoof profile apply / preview.

use egui::{Align2, RichText};

use super::i18n::{t, Msg};
use super::widgets::{
    confirm_dialog_frame, form_field_row, primary_button, section_card, subtle_button,
};
use super::CenterApp;

impl CenterApp {
    pub(super) fn panel_spoof_pipeline(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        section_card(ui, t(self.ui_lang, Msg::SpoofCardTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::SpoofBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(8.0);
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::TargetVmLabel)).small(),
                |ui| {
                    ui.add_enabled_ui(!self.net_busy, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.spoof_target_vm)
                                .desired_width(ui.available_width())
                                .hint_text(t(self.ui_lang, Msg::HintTargetVm)),
                        );
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
        });

        if self.pending_spoof_confirm_apply {
            self.show_spoof_confirm_window(ctx, ui);
        }
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
