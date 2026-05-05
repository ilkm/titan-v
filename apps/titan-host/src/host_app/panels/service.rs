use egui::RichText;
use titan_common::UiLang;

use crate::titan_egui_widgets::{form_field_row, primary_button_large, section_card};
use crate::titan_i18n::{self as i18n, Msg};

use crate::host_app::model::HostApp;

impl HostApp {
    pub(crate) fn panel_service(&mut self, ui: &mut egui::Ui) {
        let lang = self.persist.ui_lang;
        self.panel_service_env_hint(ui);
        ui.add_space(6.0);
        section_card(ui, i18n::t(lang, Msg::HpSectionControlPlane), |ui| {
            self.panel_service_listen_field(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionLanAnnounce), |ui| {
            self.panel_service_lan_checkbox(ui, lang);
            self.panel_service_lan_port_rows(ui, lang);
            self.panel_service_periodic_row(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionIdentity), |ui| {
            self.panel_service_identity_fields(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionVmStorage), |ui| {
            self.panel_service_vm_root_row(ui, lang);
        });
        ui.add_space(8.0);
        self.panel_service_actions(ui, lang);
    }

    fn panel_service_env_hint(&self, ui: &mut egui::Ui) {
        if let Some(ref h) = self.env_listen_hint {
            ui.label(RichText::new(h).small().weak());
        }
    }

    fn panel_service_listen_field(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpListen)).small(),
            |ui| {
                let w = ui.available_width().max(160.0);
                ui.add(egui::TextEdit::singleline(&mut self.persist.listen).desired_width(w));
            },
        );
    }

    fn panel_service_lan_checkbox(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.checkbox(
            &mut self.persist.announce_enabled,
            i18n::t(lang, Msg::HpAnnounce),
        );
        ui.add_space(6.0);
    }

    fn panel_service_lan_port_rows(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpPollPort)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.persist.center_poll_listen_port)
                        .speed(1.0)
                        .range(1..=65535),
                );
            },
        );
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpRegPort)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.persist.center_register_udp_port)
                        .speed(1.0)
                        .range(1..=65535),
                );
            },
        );
    }

    fn panel_service_periodic_row(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        let mut periodic = self.persist.announce_periodic_secs.unwrap_or(0);
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpPeriodic)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut periodic)
                        .speed(1.0)
                        .range(0..=86400),
                );
            },
        );
        self.persist.announce_periodic_secs = if periodic > 0 { Some(periodic) } else { None };
    }

    fn panel_service_identity_fields(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpPublicAddr)).small(),
            |ui| {
                let w = ui.available_width().max(160.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.persist.public_addr_override)
                        .desired_width(w),
                );
            },
        );
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpLabelOverride)).small(),
            |ui| {
                let w = ui.available_width().max(160.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.persist.label_override).desired_width(w),
                );
            },
        );
    }

    fn panel_service_vm_root_row(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpVmRootDir)).small(),
            |ui| {
                let w = ui.available_width().max(200.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.persist.vm_root_directory)
                        .desired_width(w)
                        .hint_text(i18n::t(lang, Msg::HpVmRootDirHint)),
                );
            },
        );
    }

    fn panel_service_actions(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        if primary_button_large(ui, i18n::t(lang, Msg::HpSaveRestart), true).clicked() {
            self.start_serve();
        }
        if !self.status_line.is_empty() {
            ui.add_space(10.0);
            let muted = ui.visuals().widgets.inactive.text_color();
            ui.label(RichText::new(&self.status_line).small().color(muted));
        }
    }
}
