//! Center settings → mTLS section: local fingerprint readout + trusted hosts list.

use egui::RichText;

use crate::app::CenterApp;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::section_card;

impl CenterApp {
    pub(crate) fn settings_mtls_section(&self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        section_card(ui, t(lang, Msg::CenterSettingsMtlsSection), |ui| {
            self.settings_mtls_local_fingerprint(ui, lang);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            self.settings_mtls_trusted_hosts(ui, lang);
        });
    }

    fn settings_mtls_local_fingerprint(&self, ui: &mut egui::Ui, lang: UiLang) {
        let muted = ui.visuals().widgets.inactive.text_color();
        ui.label(
            RichText::new(t(lang, Msg::CenterSettingsLocalFingerprint))
                .small()
                .color(muted),
        );
        ui.label(
            RichText::new(&self.center_security.identity.spki_sha256_hex)
                .monospace()
                .small(),
        );
    }

    fn settings_mtls_trusted_hosts(&self, ui: &mut egui::Ui, lang: UiLang) {
        let muted = ui.visuals().widgets.inactive.text_color();
        ui.label(
            RichText::new(t(lang, Msg::CenterSettingsTrustedHosts))
                .small()
                .color(muted),
        );
        let trust = self.center_security.trust.list();
        if trust.is_empty() {
            ui.label(
                RichText::new(t(lang, Msg::CenterSettingsNoTrustedHosts))
                    .small()
                    .color(muted),
            );
            return;
        }
        for entry in &trust {
            ui.label(
                RichText::new(format!(
                    "{} — {} ({})",
                    entry.label, entry.fingerprint, entry.source
                ))
                .small()
                .color(muted),
            );
        }
    }
}
