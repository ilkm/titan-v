//! Resource monitor: high-level device and window counts.

use egui::RichText;
use titan_common::VmPowerState;

use super::constants::CONTENT_COLUMN_GAP;
use super::i18n::{t, Msg, UiLang};
use super::widgets::section_card;
use super::CenterApp;

impl CenterApp {
    pub(super) fn panel_resource_monitor(&self, ui: &mut egui::Ui) {
        let inner = ui.available_width();
        let gap = CONTENT_COLUMN_GAP;
        let half = ((inner - gap).max(0.0)) * 0.5;

        let dev_total = self.endpoints.len();
        let dev_on = self
            .endpoints
            .iter()
            .filter(|e| e.last_known_online)
            .count();
        let dev_off = dev_total.saturating_sub(dev_on);

        let win_total = self.inventory_slice().len();
        let win_on = self
            .inventory_slice()
            .iter()
            .filter(|v| v.state == VmPowerState::Running)
            .count();
        let win_off = win_total.saturating_sub(win_on);

        ui.horizontal(|ui| {
            ui.set_min_width(inner);
            ui.vertical(|ui| {
                ui.set_width(half);
                monitor_stat_card(
                    ui,
                    self.ui_lang,
                    t(self.ui_lang, Msg::MonitorCardDevices),
                    dev_total,
                    dev_on,
                    dev_off,
                    Some(t(self.ui_lang, Msg::MonitorDevicesScopeHint)),
                );
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(half);
                monitor_stat_card(
                    ui,
                    self.ui_lang,
                    t(self.ui_lang, Msg::MonitorCardWindows),
                    win_total,
                    win_on,
                    win_off,
                    Some(t(self.ui_lang, Msg::MonitorWindowsScopeHint)),
                );
            });
        });
    }
}

fn monitor_stat_card(
    ui: &mut egui::Ui,
    lang: UiLang,
    title: &str,
    total: usize,
    online: usize,
    offline: usize,
    hint: Option<&'static str>,
) {
    section_card(ui, title, |ui| {
        stat_row(ui, t(lang, Msg::MonitorStatTotal), total);
        ui.add_space(6.0);
        stat_row(ui, t(lang, Msg::MonitorStatOnline), online);
        ui.add_space(4.0);
        stat_row(ui, t(lang, Msg::MonitorStatOffline), offline);
        if let Some(h) = hint {
            ui.add_space(10.0);
            ui.label(
                RichText::new(h)
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
        }
    });
}

fn stat_row(ui: &mut egui::Ui, label: &'static str, value: usize) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{value}"))
                    .size(16.0)
                    .strong()
                    .monospace()
                    .color(ui.visuals().strong_text_color()),
            );
        });
    });
}
