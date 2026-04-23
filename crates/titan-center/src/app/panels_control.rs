//! Connection, discovery, guest agent, and session panels.

use egui::{Frame, Margin, RichText, ScrollArea, Stroke};

use super::constants::{ACCENT, ALERT_PANEL_FILL, ERR_ROSE, OK_GREEN};
use super::i18n::{t, Msg};
use super::widgets::{form_field_row, primary_button, section_card, subtle_button};
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
            let sep = ui.separator();
            self.header_sep_center_x = sep.rect.center().x;
            let spare = (ui.available_width() - 40.0).max(0.0);
            if spare > 0.0 {
                ui.add_space(spare);
            }
            let gear = subtle_button(ui, "⚙", true);
            let gear = gear.on_hover_text(t(lang, Msg::SettingsTooltip));
            if gear.clicked() {
                self.settings_open = !self.settings_open;
            }
        });
    }

    fn panel_connection_inventory(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::ConnectionCardTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::ConnectionM2Hint))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(4.0);
            ui.add_enabled_ui(!self.net_busy, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.control_addr)
                        .desired_width(f32::INFINITY)
                        .hint_text(t(self.ui_lang, Msg::HintControlAddr)),
                );
            });
            ui.horizontal(|ui| {
                if subtle_button(ui, t(self.ui_lang, Msg::BtnSyncHost), !self.net_busy).clicked() {
                    if let Some(ep) = self.endpoints.get(self.selected_host) {
                        self.control_addr = ep.addr.clone();
                    }
                }
                let can_list = !self.net_busy && !self.control_addr.trim().is_empty();
                if subtle_button(ui, t(self.ui_lang, Msg::BtnListVms), can_list).clicked() {
                    self.spawn_list_vms();
                }
            });
            ui.add_space(6.0);
            ui.checkbox(
                &mut self.list_vms_auto_refresh,
                t(self.ui_lang, Msg::ChkAutoRefresh),
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::PollIntervalLabel)).small(),
                |ui| {
                    ui.add(egui::DragValue::new(&mut self.list_vms_poll_secs).range(5..=600));
                },
            );
        });
    }

    fn panel_discovery_lan(&mut self, ui: &mut egui::Ui) {
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
        });
    }

    fn panel_guest_agent_binding(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::GuestCardTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::GuestBlurb))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::VmLabelSmall)).small(),
                |ui| {
                    ui.add_enabled_ui(!self.net_busy, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.agent_register_vm)
                                .desired_width(ui.available_width())
                                .hint_text(t(self.ui_lang, Msg::HintVmName)),
                        );
                    });
                },
            );
            form_field_row(
                ui,
                RichText::new(t(self.ui_lang, Msg::AgentLabelSmall)).small(),
                |ui| {
                    ui.add_enabled_ui(!self.net_busy, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.agent_register_addr)
                                .desired_width(ui.available_width())
                                .hint_text(t(self.ui_lang, Msg::HintAgentAddr)),
                        );
                    });
                },
            );
            let can_reg = !self.net_busy && self.host_connected;
            if primary_button(ui, t(self.ui_lang, Msg::BtnRegisterHost), can_reg).clicked() {
                self.spawn_register_guest_agent();
            }
        });
    }

    fn panel_session_status(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::SessionTitle), |ui| {
            ui.horizontal_wrapped(|ui| {
                let can_connect = !self.net_busy && !self.host_connected;
                if primary_button(ui, t(self.ui_lang, Msg::BtnConnect), can_connect).clicked() {
                    self.spawn_connect();
                }
                let can_disconnect = !self.net_busy && self.host_connected;
                if subtle_button(ui, t(self.ui_lang, Msg::BtnDisconnect), can_disconnect).clicked()
                {
                    self.host_connected = false;
                    self.list_vms_poll_accum = 0.0;
                    self.last_capabilities.clear();
                    self.last_net_error.clear();
                    self.last_action = super::i18n::log_disconnected(self.ui_lang);
                }
                let can_ping = !self.net_busy && self.host_connected;
                if subtle_button(ui, t(self.ui_lang, Msg::BtnPing), can_ping).clicked() {
                    self.spawn_ping();
                }
                if self.net_busy {
                    ui.spinner();
                    ui.label(RichText::new(t(self.ui_lang, Msg::WaitEllipsis)).weak());
                }
            });
            ui.add_space(6.0);
            if self.host_connected {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::StatusConnected))
                        .small()
                        .strong()
                        .color(OK_GREEN),
                );
            } else {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::StatusNotConnected))
                        .small()
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
            }
            self.session_caps_and_errors(ui);
        });
    }

    fn session_caps_and_errors(&mut self, ui: &mut egui::Ui) {
        if !self.last_capabilities.is_empty() {
            ui.add_space(8.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::CapsSnapshotTitle))
                    .small()
                    .strong()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            Frame::NONE
                .fill(ui.visuals().extreme_bg_color)
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.label(RichText::new(&self.last_capabilities).monospace().small());
                    });
                });
        }
        if !self.last_net_error.is_empty() {
            ui.add_space(6.0);
            Frame::NONE
                .fill(ALERT_PANEL_FILL)
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(Margin::same(8))
                .stroke(Stroke::new(0.75, ERR_ROSE.linear_multiply(0.45)))
                .show(ui, |ui| {
                    ui.colored_label(ERR_ROSE, &self.last_net_error);
                });
        }
    }

    /// Session, guest binding, and caps (control address lives under **Settings**).
    pub(super) fn panel_device_list(&mut self, ui: &mut egui::Ui) {
        self.panel_guest_agent_binding(ui);
        self.panel_session_status(ui);
    }

    /// Settings: M2, discovery, host catalog (YouTube-style), guest binding, session.
    pub(super) fn panel_settings_host(&mut self, ui: &mut egui::Ui) {
        self.panel_connection_inventory(ui);
        self.panel_discovery_lan(ui);
        self.panel_hosts(ui);
        self.panel_device_list(ui);
    }

    /// Device management tab: no saved hosts → only「暂无数据」; otherwise redirect to Settings.
    pub(super) fn panel_device_management_redirect(&mut self, ui: &mut egui::Ui) {
        if self.endpoints.is_empty() {
            ui.add_space(48.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::NoDataShort))
                        .size(15.0)
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
            });
            return;
        }
        section_card(ui, t(self.ui_lang, Msg::NavConnect), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::DeviceMgmtNavHint))
                    .small()
                    .line_height(Some(20.0))
                    .color(ui.visuals().weak_text_color()),
            );
        });
    }
}
