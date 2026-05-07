use std::time::Duration;

use egui::RichText;
use titan_common::UiLang;

use crate::titan_egui_widgets::{form_field_row, primary_button_large, section_card};
use crate::titan_i18n::{self as i18n, Msg};

use crate::host_app::model::HostApp;

const PAIRING_WINDOW: Duration = Duration::from_secs(5 * 60);

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
            self.panel_service_lan_bind_iface_row(ui, lang);
            self.panel_service_periodic_row(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionIdentity), |ui| {
            self.panel_service_identity_fields(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionVmStorage), |ui| {
            self.panel_service_vm_root_row(ui, lang);
        });
        section_card(ui, i18n::t(lang, Msg::HpSectionMtlsPairing), |ui| {
            self.panel_service_mtls_pairing(ui, lang);
        });
        ui.add_space(8.0);
        self.panel_service_actions(ui, lang);
    }

    fn panel_service_mtls_pairing(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        self.panel_service_mtls_fingerprint_row(ui, lang);
        ui.add_space(6.0);
        self.panel_service_mtls_pairing_controls(ui, lang);
        ui.add_space(8.0);
        self.panel_service_mtls_trust_list(ui, lang);
    }

    fn panel_service_mtls_fingerprint_row(&self, ui: &mut egui::Ui, lang: UiLang) {
        let muted = ui.visuals().widgets.inactive.text_color();
        let fp = &self.host_security.identity.spki_sha256_hex;
        let label = i18n::t(lang, Msg::HpQuicFingerprintLabel);
        ui.label(RichText::new(format!("{label}: {fp}")).small().color(muted));
    }

    fn panel_service_mtls_pairing_controls(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        let snap = self.host_security.pairing.snapshot();
        if snap.open {
            let secs = snap.ttl_remaining_ms / 1000;
            ui.label(RichText::new(i18n::hp_quic_pairing_remaining(lang, secs)).small());
            if primary_button_large(ui, i18n::t(lang, Msg::HpQuicPairingClose), true).clicked() {
                self.host_security.pairing.close();
            }
        } else if primary_button_large(ui, i18n::t(lang, Msg::HpQuicPairingOpenBtn), true).clicked()
        {
            self.host_security.pairing.open(PAIRING_WINDOW);
        }
    }

    fn panel_service_mtls_trust_list(&self, ui: &mut egui::Ui, lang: UiLang) {
        let muted = ui.visuals().widgets.inactive.text_color();
        let trusted = self.host_security.trust.list();
        if trusted.is_empty() {
            ui.label(
                RichText::new(i18n::t(lang, Msg::HpQuicNoTrustedCenters))
                    .small()
                    .color(muted),
            );
            return;
        }
        ui.label(RichText::new(i18n::t(lang, Msg::HpQuicTrustedCentersHeader)).small());
        for entry in &trusted {
            ui.label(
                RichText::new(format!(
                    "  {} — {} ({})",
                    entry.label, entry.fingerprint, entry.source
                ))
                .small()
                .color(muted),
            );
        }
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

    fn panel_service_lan_bind_iface_row(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        let rows = self.panel_service_lan_bind_rows();
        self.panel_service_lan_bind_field(ui, lang, &rows);
    }

    fn panel_service_lan_bind_rows(&mut self) -> Vec<crate::serve::LanIpv4Row> {
        let rows = crate::serve::list_physical_lan_ipv4_rows();
        self.persist.normalize_lan_bind_ipv4_with_rows(&rows);
        rows
    }

    fn panel_service_lan_bind_field(
        &mut self,
        ui: &mut egui::Ui,
        lang: UiLang,
        rows: &[crate::serve::LanIpv4Row],
    ) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpLanBindIface)).small(),
            |ui| {
                if rows.is_empty() {
                    ui.label(RichText::new(i18n::t(lang, Msg::HpLanBindIfaceNone)).weak());
                    return;
                }
                egui::ComboBox::from_id_salt("hp_lan_bind_iface")
                    .selected_text(self.lan_bind_selected_text(lang, rows))
                    .show_ui(ui, |ui| {
                        for row in rows {
                            let ip = row.ip.to_string();
                            let label = format!("{} ({})", row.iface, ip);
                            ui.selectable_value(&mut self.persist.lan_bind_ipv4, ip, label);
                        }
                    });
            },
        );
    }

    fn lan_bind_selected_text(&self, lang: UiLang, rows: &[crate::serve::LanIpv4Row]) -> String {
        if rows.is_empty() || self.persist.lan_bind_ipv4.trim().is_empty() {
            return i18n::t(lang, Msg::HpLanBindIfaceNone).to_string();
        }
        rows.iter()
            .find(|row| row.ip.to_string() == self.persist.lan_bind_ipv4)
            .map(|row| format!("{} ({})", row.iface, row.ip))
            .unwrap_or_else(|| self.persist.lan_bind_ipv4.clone())
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
