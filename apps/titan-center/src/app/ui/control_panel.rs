//! Top bar, LAN discovery, and LAN host registration (settings tab body).

use egui::{RichText, ScrollArea};

use crate::app::CenterApp;
use crate::app::constants::ACCENT;
use crate::app::i18n::{Msg, t};
use crate::app::ui::widgets::{form_field_row, section_card, subtle_button};

impl CenterApp {
    pub(crate) fn top_status_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(
                RichText::new(t(lang, Msg::BrandTitle))
                    .strong()
                    .size(19.0)
                    .extra_letter_spacing(0.25)
                    .color(ACCENT),
            );
            let spare = (ui.available_width() - 40.0).max(0.0);
            if spare > 0.0 {
                ui.add_space(spare);
            }
            let lang_btn = subtle_button(ui, "🌐", true);
            let lang_btn = lang_btn.on_hover_text(t(lang, Msg::SettingsTooltip));
            self.settings_lang_btn_rect = Some(lang_btn.rect);
            if lang_btn.clicked() {
                self.settings_open = !self.settings_open;
            }
        });
    }

    fn settings_discovery_section(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        section_card(ui, t(lang, Msg::DiscoveryTitle), |ui| {
            self.settings_discovery_udp_controls(ui, lang);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            self.settings_discovery_bind_block(ui, lang);
        });
    }

    fn settings_discovery_udp_controls(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        ui.checkbox(
            &mut self.discovery_broadcast,
            t(lang, Msg::DiscoveryCheckbox),
        );
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::IntervalLabel)).small(),
            |ui| {
                ui.add(egui::DragValue::new(&mut self.discovery_interval_secs).range(1..=600));
            },
        );
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::UdpPortLabel)).small(),
            |ui| {
                ui.add(egui::DragValue::new(&mut self.discovery_udp_port).range(1..=65535));
            },
        );
    }

    fn settings_discovery_bind_block(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        ui.label(
            RichText::new(t(lang, Msg::DiscoveryBindBlurb))
                .small()
                .color(ui.visuals().widgets.inactive.text_color()),
        );
        ui.add_space(6.0);
        self.settings_discovery_bind_toolbar(ui, lang);
        ui.add_space(6.0);
        ui.label(
            RichText::new(t(lang, Msg::DiscoveryBindScrollHint))
                .small()
                .strong(),
        );
        self.settings_discovery_bind_list(ui, lang);
    }

    fn settings_discovery_bind_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        let total = self.discovery_if_rows.len();
        let selected = self.discovery_bind_ipv4s.len();
        let can_select_all = total > 0 && selected < total;
        ui.horizontal(|ui| {
            if subtle_button(ui, t(lang, Msg::DiscoveryRefreshIfaces), true).clicked() {
                self.settings_refresh_discovery_ifaces();
            }
            if subtle_button(
                ui,
                t(lang, Msg::DiscoveryClearBindIps),
                !self.discovery_bind_ipv4s.is_empty(),
            )
            .clicked()
            {
                self.discovery_bind_ipv4s.clear();
            }
            if subtle_button(ui, t(lang, Msg::DiscoverySelectAllBindIps), can_select_all).clicked()
            {
                self.settings_select_all_discovery_bind_ips();
            }
        });
    }

    fn settings_refresh_discovery_ifaces(&mut self) {
        self.refresh_discovery_ifaces_now();
    }

    fn settings_select_all_discovery_bind_ips(&mut self) {
        self.discovery_bind_ipv4s = self
            .discovery_if_rows
            .iter()
            .map(|row| row.ip.to_string())
            .collect();
    }

    fn settings_discovery_bind_list(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        if self.discovery_if_rows.is_empty() {
            ui.add_space(4.0);
            ui.label(
                RichText::new(t(lang, Msg::DiscoveryNoIpv4Ifaces))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            return;
        }
        ui.add_space(4.0);
        ScrollArea::vertical()
            .id_salt("discovery_bind_ipv4_list")
            .max_height(160.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for row in &self.discovery_if_rows {
                    let ip_s = row.ip.to_string();
                    let mut on = self.discovery_bind_ipv4s.contains(&ip_s);
                    let label = format!("{} — {}", row.ip, row.iface);
                    if ui.checkbox(&mut on, label).changed() {
                        Self::discovery_toggle_bind_ip(&mut self.discovery_bind_ipv4s, &ip_s, on);
                    }
                }
            });
    }

    fn discovery_toggle_bind_ip(bind_ips: &mut Vec<String>, ip_s: &str, on: bool) {
        if on {
            if !bind_ips.iter().any(|e| e == ip_s) {
                bind_ips.push(ip_s.to_string());
            }
        } else {
            bind_ips.retain(|x| x != ip_s);
        }
    }

    fn settings_host_collect_section(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        section_card(ui, t(lang, Msg::HostCollectTitle), |ui| {
            ui.label(
                RichText::new(t(lang, Msg::HostCollectBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            ui.checkbox(
                &mut self.host_collect_broadcast,
                t(lang, Msg::HostCollectCheckbox),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            self.settings_host_collect_port_fields(ui, lang);
        });
    }

    fn settings_host_collect_port_fields(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HostCollectIntervalLabel)).small(),
            |ui| {
                ui.add(egui::DragValue::new(&mut self.host_collect_interval_secs).range(1..=600));
            },
        );
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HostCollectPollPortLabel)).small(),
            |ui| {
                ui.add(egui::DragValue::new(&mut self.host_collect_poll_udp_port).range(1..=65535));
            },
        );
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HostCollectRegisterPortLabel)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.host_collect_register_udp_port).range(1..=65535),
                );
            },
        );
        let _ = lang;
    }

    /// Settings: LAN discovery + LAN host registration + mTLS trust readout.
    pub(crate) fn panel_settings_host(&mut self, ui: &mut egui::Ui) {
        self.refresh_discovery_iface_rows(ui);
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            self.settings_discovery_section(ui);
            self.settings_host_collect_section(ui);
            self.settings_mtls_section(ui);
        });
    }

    /// Device management tab: large per-host cards (no titled host-grid card).
    pub(crate) fn panel_device_management_redirect(&mut self, ui: &mut egui::Ui) {
        self.panel_device_management(ui);
    }
}
