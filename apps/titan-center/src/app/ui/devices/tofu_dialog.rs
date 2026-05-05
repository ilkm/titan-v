//! Trust-on-first-use prompt: shown when a manually-added Host's QUIC fingerprint is unknown.
//!
//! Behaviour: confirm → write fingerprint to the Center trust store and re-trigger
//! `spawn_add_host_verify`; cancel → drop the prompt without persisting trust.

use egui::{Align, CornerRadius, Frame, Layout, Margin, RichText, Stroke, Vec2};

use super::helpers::{
    ADD_HOST_DLG_BODY, ADD_HOST_DLG_LABEL, ADD_HOST_DLG_MUTED, ADD_HOST_ERR_BG,
    ADD_HOST_ERR_BORDER, ADD_HOST_ERR_TEXT,
};
use crate::app::CenterApp;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::{
    OpaqueFrameSource, primary_button_large, show_opaque_modal, subtle_button_large,
};

const DIALOG_INNER: Vec2 = Vec2::new(480.0, 360.0);

impl CenterApp {
    pub(crate) fn show_tofu_dialog(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        if self.tofu_pending.is_none() {
            return;
        }
        let mut win_open = true;
        let ctx = ui.ctx().clone();
        let mut confirm = false;
        let mut cancel = false;
        let prompt = self.tofu_pending.clone();
        show_opaque_modal(
            &ctx,
            egui::Id::new("titan_center_tofu_dialog"),
            t(lang, Msg::CenterTofuDialogTitle),
            &mut win_open,
            DIALOG_INNER,
            OpaqueFrameSource::Ui(ui),
            |ui| Self::tofu_dialog_body(ui, lang, &prompt, &mut confirm, &mut cancel),
        );
        if confirm {
            self.confirm_tofu_pending();
        } else if cancel || !win_open {
            self.dismiss_tofu_pending();
        }
    }

    fn tofu_dialog_body(
        ui: &mut egui::Ui,
        lang: UiLang,
        prompt: &Option<crate::app::TofuPrompt>,
        confirm: &mut bool,
        cancel: &mut bool,
    ) {
        let Some(prompt) = prompt.as_ref() else {
            return;
        };
        let full_w = ui.available_width();
        ui.set_width(full_w);
        ui.spacing_mut().item_spacing.y = 0.0;
        Self::tofu_dialog_header(ui, lang);
        ui.add_space(14.0);
        Self::tofu_dialog_field(ui, lang, Msg::CenterTofuHostLabel, &prompt.host_addr);
        ui.add_space(10.0);
        Self::tofu_dialog_field(
            ui,
            lang,
            Msg::CenterTofuFingerprintLabel,
            &prompt.fingerprint,
        );
        ui.add_space(14.0);
        Self::tofu_dialog_warning(ui, lang);
        ui.add_space(20.0);
        Self::tofu_dialog_actions(ui, lang, full_w, confirm, cancel);
    }

    fn tofu_dialog_header(ui: &mut egui::Ui, lang: UiLang) {
        ui.add(
            egui::Label::new(
                RichText::new(t(lang, Msg::CenterTofuDialogSubtitle))
                    .size(12.5)
                    .line_height(Some(18.0))
                    .color(ADD_HOST_DLG_MUTED),
            )
            .wrap(),
        );
    }

    fn tofu_dialog_field(ui: &mut egui::Ui, lang: UiLang, label: Msg, value: &str) {
        ui.label(
            RichText::new(t(lang, label))
                .size(13.0)
                .strong()
                .color(ADD_HOST_DLG_LABEL),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(value)
                .monospace()
                .size(12.5)
                .color(ADD_HOST_DLG_BODY),
        );
    }

    fn tofu_dialog_warning(ui: &mut egui::Ui, lang: UiLang) {
        Frame::NONE
            .fill(ADD_HOST_ERR_BG)
            .stroke(Stroke::new(1.0, ADD_HOST_ERR_BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(t(lang, Msg::CenterTofuWarning))
                            .size(12.5)
                            .line_height(Some(18.0))
                            .color(ADD_HOST_ERR_TEXT),
                    )
                    .wrap(),
                );
            });
    }

    fn tofu_dialog_actions(
        ui: &mut egui::Ui,
        lang: UiLang,
        full_w: f32,
        confirm: &mut bool,
        cancel: &mut bool,
    ) {
        ui.allocate_ui_with_layout(
            egui::vec2(full_w, 48.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                if primary_button_large(ui, t(lang, Msg::CenterTofuConfirm), true).clicked() {
                    *confirm = true;
                }
                if subtle_button_large(ui, t(lang, Msg::BtnCancel), true).clicked() {
                    *cancel = true;
                }
            },
        );
    }
}
