//! Resource monitor: high-level device and window counts.

use egui::RichText;
use titan_common::VmPowerState;

use crate::app::constants::CONTENT_COLUMN_GAP;
use crate::app::i18n::{t, Msg, UiLang};
use crate::app::ui::widgets::section_card;
use crate::app::CenterApp;

impl CenterApp {
    pub(crate) fn panel_resource_monitor(&self, ui: &mut egui::Ui) {
        let inner = ui.available_width();
        let gap = CONTENT_COLUMN_GAP;
        let half = ((inner - gap).max(0.0)) * 0.5;
        let dev = self.monitor_device_totals();
        let win = self.monitor_window_totals();
        paint_monitor_stat_columns(ui, self.ui_lang, inner, gap, half, dev, win);
    }

    fn monitor_device_totals(&self) -> (usize, usize, usize) {
        let total = self.endpoints.len();
        let on = self
            .endpoints
            .iter()
            .filter(|e| e.last_known_online)
            .count();
        (total, on, total.saturating_sub(on))
    }

    fn monitor_window_totals(&self) -> (usize, usize, usize) {
        let slice = self.inventory_slice();
        let total = slice.len();
        let on = slice
            .iter()
            .filter(|v| v.state == VmPowerState::Running)
            .count();
        (total, on, total.saturating_sub(on))
    }
}

fn paint_monitor_device_column(
    ui: &mut egui::Ui,
    lang: UiLang,
    half: f32,
    dev: (usize, usize, usize),
) {
    ui.vertical(|ui| {
        ui.set_width(half);
        monitor_stat_card(
            ui,
            lang,
            t(lang, Msg::MonitorCardDevices),
            dev.0,
            dev.1,
            dev.2,
            Some(t(lang, Msg::MonitorDevicesScopeHint)),
        );
    });
}

fn paint_monitor_windows_column(
    ui: &mut egui::Ui,
    lang: UiLang,
    half: f32,
    win: (usize, usize, usize),
) {
    ui.vertical(|ui| {
        ui.set_width(half);
        monitor_stat_card(
            ui,
            lang,
            t(lang, Msg::MonitorCardWindows),
            win.0,
            win.1,
            win.2,
            Some(t(lang, Msg::MonitorWindowsScopeHint)),
        );
    });
}

fn paint_monitor_stat_columns(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner: f32,
    gap: f32,
    half: f32,
    dev: (usize, usize, usize),
    win: (usize, usize, usize),
) {
    ui.horizontal(|ui| {
        ui.set_min_width(inner);
        paint_monitor_device_column(ui, lang, half, dev);
        ui.add_space(gap);
        paint_monitor_windows_column(ui, lang, half, win);
    });
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
