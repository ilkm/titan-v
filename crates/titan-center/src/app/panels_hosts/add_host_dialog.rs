use std::net::Ipv4Addr;
use std::str::FromStr;

use egui::{
    Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, RichText, Stroke, TextStyle, Vec2,
};

use super::super::i18n::{t, Msg};
use super::super::widgets::{
    dialog_underline_text_row, opaque_dialog_frame, primary_button_large, subtle_button_large,
};
use super::super::CenterApp;
use super::helpers::{
    ADD_HOST_DLG_BODY, ADD_HOST_DLG_LABEL, ADD_HOST_DLG_MUTED, ADD_HOST_ERR_BG,
    ADD_HOST_ERR_BORDER, ADD_HOST_ERR_TEXT,
};

impl CenterApp {
    pub(crate) fn show_add_host_dialog(
        &mut self,
        ui: &mut egui::Ui,
        lang: super::super::i18n::UiLang,
    ) {
        if !self.add_host_dialog_open {
            return;
        }
        let mut win_open = self.add_host_dialog_open;
        let mut force_close = false;
        let ctx = ui.ctx().clone();
        self.add_host_dialog_open_window(ui, &ctx, lang, &mut win_open, &mut force_close);
        if force_close {
            win_open = false;
        }
        self.add_host_dialog_open = win_open;
        if !self.add_host_dialog_open {
            self.add_host_dialog_err.clear();
            self.invalidate_add_host_probe();
        }
    }

    fn add_host_dialog_open_window(
        &mut self,
        outer_ui: &mut egui::Ui,
        ctx: &egui::Context,
        lang: super::super::i18n::UiLang,
        win_open: &mut bool,
        force_close: &mut bool,
    ) {
        let title = t(lang, Msg::AddHostDialogTitle);
        const DIALOG_INNER: Vec2 = Vec2::new(440.0, 312.0);
        egui::Window::new(title)
            .id(egui::Id::new("titan_center_add_host_dialog"))
            .frame(opaque_dialog_frame(outer_ui))
            .open(win_open)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .fade_in(false)
            .fade_out(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(DIALOG_INNER)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                self.add_host_dialog_window_body(ui, lang, force_close);
            });
    }

    fn add_host_dialog_window_body(
        &mut self,
        ui: &mut egui::Ui,
        lang: super::super::i18n::UiLang,
        force_close: &mut bool,
    ) {
        let full_w = ui.available_width();
        ui.set_width(full_w);
        ui.spacing_mut().item_spacing.y = 0.0;
        Self::add_host_dialog_subtitle(ui, lang);
        ui.add_space(16.0);
        self.add_host_dialog_ip_field(ui, lang);
        ui.add_space(14.0);
        self.add_host_dialog_port_field(ui, lang);
        ui.add_space(12.0);
        self.add_host_dialog_error_banner(ui);
        self.add_host_dialog_verify_busy(ui, lang);
        ui.add_space(20.0);
        self.add_host_dialog_action_row(ui, lang, full_w, force_close);
    }

    fn add_host_dialog_subtitle(ui: &mut egui::Ui, lang: super::super::i18n::UiLang) {
        ui.add(
            egui::Label::new(
                RichText::new(t(lang, Msg::AddHostDialogSubtitle))
                    .size(12.5)
                    .line_height(Some(18.0))
                    .color(ADD_HOST_DLG_MUTED),
            )
            .wrap(),
        );
    }

    fn add_host_dialog_ip_field(&mut self, ui: &mut egui::Ui, lang: super::super::i18n::UiLang) {
        ui.label(
            RichText::new(t(lang, Msg::AddHostIpLabel))
                .size(13.0)
                .strong()
                .color(ADD_HOST_DLG_LABEL),
        );
        ui.add_space(6.0);
        dialog_underline_text_row(ui, |ui| {
            egui::TextEdit::singleline(&mut self.add_host_dialog_ip)
                .frame(false)
                .background_color(Color32::TRANSPARENT)
                .margin(Margin::symmetric(0, 8))
                .desired_width(ui.available_width())
                .font(TextStyle::Monospace)
                .hint_text(RichText::new("192.168.1.1").color(ADD_HOST_DLG_MUTED))
                .text_color(ADD_HOST_DLG_BODY)
                .show(ui)
        });
    }

    fn add_host_dialog_port_field(&mut self, ui: &mut egui::Ui, lang: super::super::i18n::UiLang) {
        ui.label(
            RichText::new(t(lang, Msg::AddHostPortLabel))
                .size(13.0)
                .strong()
                .color(ADD_HOST_DLG_LABEL),
        );
        ui.add_space(6.0);
        dialog_underline_text_row(ui, |ui| {
            egui::TextEdit::singleline(&mut self.add_host_dialog_port)
                .frame(false)
                .background_color(Color32::TRANSPARENT)
                .margin(Margin::symmetric(0, 8))
                .desired_width(ui.available_width())
                .font(TextStyle::Monospace)
                .hint_text(RichText::new("7788").color(ADD_HOST_DLG_MUTED))
                .text_color(ADD_HOST_DLG_BODY)
                .show(ui)
        });
    }

    fn add_host_dialog_error_banner(&self, ui: &mut egui::Ui) {
        if self.add_host_dialog_err.is_empty() {
            return;
        }
        Frame::NONE
            .fill(ADD_HOST_ERR_BG)
            .stroke(Stroke::new(1.0, ADD_HOST_ERR_BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(&self.add_host_dialog_err)
                            .size(12.5)
                            .line_height(Some(18.0))
                            .color(ADD_HOST_ERR_TEXT),
                    )
                    .wrap(),
                );
            });
    }

    fn add_host_dialog_verify_busy(&self, ui: &mut egui::Ui, lang: super::super::i18n::UiLang) {
        if !self.add_host_verify_busy {
            return;
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new(t(lang, Msg::AddHostVerifying))
                .size(13.0)
                .color(ADD_HOST_DLG_MUTED),
        );
    }

    fn add_host_dialog_action_row(
        &mut self,
        ui: &mut egui::Ui,
        lang: super::super::i18n::UiLang,
        full_w: f32,
        force_close: &mut bool,
    ) {
        ui.allocate_ui_with_layout(
            egui::vec2(full_w, 48.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                self.add_host_dialog_confirm_or_err(ui, lang);
                if subtle_button_large(ui, t(lang, Msg::BtnCancel), true).clicked() {
                    *force_close = true;
                    self.add_host_dialog_err.clear();
                }
            },
        );
    }

    fn add_host_dialog_confirm_or_err(
        &mut self,
        ui: &mut egui::Ui,
        lang: super::super::i18n::UiLang,
    ) {
        if !primary_button_large(ui, t(lang, Msg::AddHostConfirm), !self.add_host_verify_busy)
            .clicked()
        {
            return;
        }
        let ip_s = self.add_host_dialog_ip.trim();
        let port_s = self.add_host_dialog_port.trim();
        let ip_ok = Ipv4Addr::from_str(ip_s).ok();
        let port_ok: Option<u16> = port_s.parse().ok().filter(|&p| p > 0);
        if let (Some(ip), Some(port)) = (ip_ok, port_ok) {
            let addr = format!("{ip}:{port}");
            self.spawn_add_host_verify(addr);
            self.add_host_dialog_err.clear();
        } else {
            self.add_host_dialog_err = t(lang, Msg::AddHostInvalidHint).to_string();
        }
    }
}
