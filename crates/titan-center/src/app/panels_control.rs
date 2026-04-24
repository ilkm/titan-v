//! Top bar, LAN discovery, and LAN host registration (settings tab body).

use egui::{RichText, ScrollArea};

use super::constants::ACCENT;
use super::i18n::{t, Msg};
use super::widgets::{form_field_row, section_card, subtle_button};
use super::CenterApp;

impl CenterApp {
    pub(super) fn top_status_bar(&mut self, ui: &mut egui::Ui) {
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
            if lang_btn.clicked() {
                self.settings_open = !self.settings_open;
            }
        });
    }

    fn settings_discovery_section(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::DiscoveryTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::DiscoveryUdpBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            ui.checkbox(
                &mut self.discovery_broadcast,
                t(self.ui_lang, Msg::DiscoveryCheckbox),
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::IntervalLabel)).small(),
                |ui| {
                    ui.add(egui::DragValue::new(&mut self.discovery_interval_secs).range(1..=600));
                },
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::UdpPortLabel)).small(),
                |ui| {
                    ui.add(egui::DragValue::new(&mut self.discovery_udp_port).range(1..=65535));
                },
            );
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::DiscoveryBindBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if subtle_button(ui, t(self.ui_lang, Msg::DiscoveryRefreshIfaces), true).clicked() {
                    self.discovery_if_rows = super::discovery::list_lan_ipv4_rows();
                    self.discovery_if_scan_secs = ui.ctx().input(|i| i.time);
                }
                if subtle_button(
                    ui,
                    t(self.ui_lang, Msg::DiscoveryClearBindIps),
                    !self.discovery_bind_ipv4s.is_empty(),
                )
                .clicked()
                {
                    self.discovery_bind_ipv4s.clear();
                }
            });
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("discovery_ipv4_quick_add")
                .selected_text(t(self.ui_lang, Msg::DiscoveryBindQuickAdd))
                .show_ui(ui, |ui| {
                    if self.discovery_if_rows.is_empty() {
                        ui.weak(t(self.ui_lang, Msg::DiscoveryNoIpv4Ifaces));
                    }
                    for row in &self.discovery_if_rows {
                        let ip_s = row.ip.to_string();
                        let label = format!(
                            "{} — {}{}",
                            row.ip,
                            row.iface,
                            if self.discovery_bind_ipv4s.contains(&ip_s) {
                                "  ✓"
                            } else {
                                ""
                            }
                        );
                        if ui.button(label).clicked() && !self.discovery_bind_ipv4s.contains(&ip_s)
                        {
                            self.discovery_bind_ipv4s.push(ip_s);
                        }
                    }
                });
            ui.add_space(6.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::DiscoveryBindScrollHint))
                    .small()
                    .strong(),
            );
            if self.discovery_if_rows.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::DiscoveryNoIpv4Ifaces))
                        .small()
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
            } else {
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
                                if on {
                                    if !self.discovery_bind_ipv4s.contains(&ip_s) {
                                        self.discovery_bind_ipv4s.push(ip_s);
                                    }
                                } else {
                                    self.discovery_bind_ipv4s.retain(|x| x != &ip_s);
                                }
                            }
                        }
                    });
            }
        });
    }

    fn settings_host_collect_section(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::HostCollectTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::HostCollectBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            ui.checkbox(
                &mut self.host_collect_broadcast,
                t(self.ui_lang, Msg::HostCollectCheckbox),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::HostCollectIntervalLabel)).small(),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.host_collect_interval_secs).range(1..=600),
                    );
                },
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::HostCollectPollPortLabel)).small(),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.host_collect_poll_udp_port).range(1..=65535),
                    );
                },
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::HostCollectRegisterPortLabel)).small(),
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.host_collect_register_udp_port)
                            .range(1..=65535),
                    );
                },
            );
        });
    }

    /// Settings: LAN discovery + LAN host registration only.
    pub(super) fn panel_settings_host(&mut self, ui: &mut egui::Ui) {
        self.refresh_discovery_iface_rows(ui);
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            self.settings_discovery_section(ui);
            self.settings_host_collect_section(ui);
        });
    }

    /// Device management tab: large per-host cards (no titled host-grid card).
    pub(super) fn panel_device_management_redirect(&mut self, ui: &mut egui::Ui) {
        self.panel_device_management(ui);
    }
}
