use crate::titan_i18n::{self as i18n, Msg};

use crate::host_app::model::HostApp;

impl HostApp {
    pub(crate) fn panel_service(&mut self, ui: &mut egui::Ui) {
        let lang = self.persist.ui_lang;
        self.panel_service_env_hint(ui, lang);
        self.panel_service_listen_row(ui, lang);
        self.panel_service_announce_block(ui, lang);
        self.panel_service_public_and_label(ui, lang);
        ui.add_space(12.0);
        self.panel_service_save_and_status(ui, lang);
    }

    fn panel_service_env_hint(&self, ui: &mut egui::Ui, _lang: titan_common::UiLang) {
        if let Some(ref h) = self.env_listen_hint {
            ui.label(egui::RichText::new(h).weak());
            ui.add_space(4.0);
        }
    }

    fn panel_service_listen_row(&mut self, ui: &mut egui::Ui, lang: titan_common::UiLang) {
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpListen));
            ui.text_edit_singleline(&mut self.persist.listen);
        });
    }

    fn panel_service_announce_block(&mut self, ui: &mut egui::Ui, lang: titan_common::UiLang) {
        ui.checkbox(
            &mut self.persist.announce_enabled,
            i18n::t(lang, Msg::HpAnnounce),
        );
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpPollPort));
            ui.add(
                egui::DragValue::new(&mut self.persist.center_poll_listen_port)
                    .speed(1.0)
                    .range(1..=65535),
            );
        });
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpRegPort));
            ui.add(
                egui::DragValue::new(&mut self.persist.center_register_udp_port)
                    .speed(1.0)
                    .range(1..=65535),
            );
        });
        self.panel_service_announce_periodic(ui, lang);
    }

    fn panel_service_announce_periodic(&mut self, ui: &mut egui::Ui, lang: titan_common::UiLang) {
        let mut periodic = self.persist.announce_periodic_secs.unwrap_or(0);
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpPeriodic));
            if ui
                .add(
                    egui::DragValue::new(&mut periodic)
                        .speed(1.0)
                        .range(0..=86400),
                )
                .changed()
            {
                self.persist.announce_periodic_secs =
                    if periodic > 0 { Some(periodic) } else { None };
            }
        });
    }

    fn panel_service_public_and_label(&mut self, ui: &mut egui::Ui, lang: titan_common::UiLang) {
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpPublicAddr));
            ui.text_edit_singleline(&mut self.persist.public_addr_override);
        });
        ui.horizontal(|ui| {
            ui.label(i18n::t(lang, Msg::HpLabelOverride));
            ui.text_edit_singleline(&mut self.persist.label_override);
        });
    }

    fn panel_service_save_and_status(&mut self, ui: &mut egui::Ui, lang: titan_common::UiLang) {
        if ui.button(i18n::t(lang, Msg::HpSaveRestart)).clicked() {
            self.start_serve();
        }
        if !self.status_line.is_empty() {
            ui.add_space(4.0);
            ui.label(&self.status_line);
        }
    }
}
