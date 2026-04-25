//! Main window layout: top bar, side nav, central stack, settings popup, device persistence hook.

use egui::{
    Align, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextStyle, TextWrapMode,
    WidgetText,
};

use super::constants::{
    card_shadow, ACCENT, CONTENT_MAX_WIDTH, NAV_ITEM_HEIGHT, SIDEBAR_DEFAULT_WIDTH,
};
use super::device_store;
use super::i18n::{self, Msg, UiLang};
use super::persist_data::NavTab;
use super::CenterApp;

fn effective_content_width(full_w: f32) -> f32 {
    let scalable = (CONTENT_MAX_WIDTH * 1.15).max(full_w * 0.92);
    full_w.min(scalable).max(280.0)
}

impl CenterApp {
    pub(crate) fn render_top_panel(&mut self, ctx: &egui::Context) {
        let visuals = ctx.style().visuals.clone();
        let top_fill = visuals.panel_fill;
        egui::TopBottomPanel::top("status")
            .frame(
                Frame::NONE
                    .fill(top_fill)
                    .inner_margin(Margin::symmetric(22, 12))
                    .stroke(Stroke::NONE)
                    .shadow(card_shadow()),
            )
            .show_separator_line(true)
            .default_height(52.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    self.top_status_bar(ui);
                });
            });
    }

    fn side_nav_one_row(&mut self, ui: &mut egui::Ui, lang: UiLang, tab: NavTab, msg: Msg) {
        let selected = self.active_nav == tab;
        let w = ui.available_width();
        let label = i18n::t(lang, msg);
        let inactive = ui.visuals().widgets.inactive.text_color();
        let text = if selected {
            RichText::new(label).size(14.0).strong().color(ACCENT)
        } else {
            RichText::new(label).size(14.0).color(inactive)
        };
        let galley = WidgetText::from(text).into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Button,
        );
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(w, NAV_ITEM_HEIGHT), Sense::click());
        if ui.is_rect_visible(rect) {
            let y = rect.center().y - 0.5 * galley.size().y;
            ui.painter()
                .galley(egui::pos2(rect.min.x, y), galley, inactive);
        }
        if response.clicked() {
            self.active_nav = tab;
        }
    }

    fn render_side_nav_tab_buttons(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.spacing_mut().item_spacing.y = 6.0;
        let tabs = [
            (NavTab::Monitor, Msg::NavMonitor),
            (NavTab::Connect, Msg::NavConnect),
            (NavTab::HostsVms, Msg::NavHostsVms),
            (NavTab::Spoof, Msg::NavSpoof),
            (NavTab::Power, Msg::NavPower),
            (NavTab::Settings, Msg::NavSettings),
        ];
        for &(tab, msg) in &tabs {
            self.side_nav_one_row(ui, lang, tab, msg);
        }
    }

    pub(crate) fn render_side_nav(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let visuals = ctx.style().visuals.clone();
        let nav_fill = visuals.extreme_bg_color;
        egui::SidePanel::left("nav")
            .exact_width(SIDEBAR_DEFAULT_WIDTH)
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(nav_fill)
                    .inner_margin(Margin::symmetric(10, 14))
                    .stroke(Stroke::NONE),
            )
            .show_separator_line(true)
            .show(ctx, |ui| {
                self.render_side_nav_tab_buttons(ui, lang);
            });
    }

    fn central_panel_horizontal_padded(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let full_w = ui.available_width();
        let content_w = effective_content_width(full_w);
        let column_w = content_w.min(full_w);
        let side_remain = (full_w - column_w).max(0.0);
        let (left_pad, right_pad) = if self.active_nav == NavTab::Settings {
            (0.0, side_remain)
        } else {
            let half = 0.5 * side_remain;
            (half, half)
        };
        ui.horizontal(|ui| {
            ui.add_space(left_pad);
            ui.vertical(|ui| {
                ui.set_width(column_w);
                match self.active_nav {
                    NavTab::Connect => self.panel_device_management_redirect(ui),
                    NavTab::Settings => self.panel_settings_host(ui),
                    NavTab::HostsVms => self.panel_window_management(ui),
                    NavTab::Monitor => self.panel_resource_monitor(ui),
                    NavTab::Spoof => self.panel_spoof_pipeline(ui, ctx),
                    NavTab::Power => self.panel_danger(ui, ctx),
                }
            });
            ui.add_space(right_pad);
        });
    }

    pub(crate) fn render_central_panel(&mut self, ctx: &egui::Context) {
        let central_fill = ctx.style().visuals.window_fill;
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(central_fill)
                    .inner_margin(Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(self.active_nav as u8)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.central_panel_horizontal_padded(ui, ctx);
                    });
            });
    }

    fn settings_window_lang_radios(ui: &mut egui::Ui, ui_lang: &mut UiLang, lang: UiLang) {
        ui.radio_value(ui_lang, UiLang::En, i18n::t(lang, Msg::LangRadioEn));
        ui.radio_value(ui_lang, UiLang::Zh, i18n::t(lang, Msg::LangRadioZh));
        ui.add_space(12.0);
        ui.label(
            RichText::new(i18n::t(lang, Msg::SettingsMoreLangNote))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    }

    fn settings_window_sqlite_path(ui: &mut egui::Ui, lang: UiLang) {
        ui.separator();
        ui.label(
            RichText::new(i18n::t(lang, Msg::SettingsDbCaption))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        let p = device_store::registration_db_path();
        ui.label(
            RichText::new(p.display().to_string())
                .monospace()
                .size(11.0),
        );
        ui.label(
            RichText::new(i18n::t(lang, Msg::SettingsDbHint))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    }

    pub(crate) fn render_settings_window(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let mut close_clicked = false;
        egui::Window::new(i18n::t(lang, Msg::SettingsTitle))
            .open(&mut self.settings_open)
            .collapsible(false)
            .resizable(false)
            .default_pos(ctx.screen_rect().right_top() + egui::vec2(-256.0, 48.0))
            .default_width(420.0)
            .show(ctx, |ui| {
                Self::settings_window_lang_radios(ui, &mut self.ui_lang, lang);
                Self::settings_window_sqlite_path(ui, lang);
                ui.add_space(8.0);
                if ui.button(i18n::t(lang, Msg::SettingsClose)).clicked() {
                    close_clicked = true;
                }
            });
        if close_clicked {
            self.settings_open = false;
        }
    }

    /// Persist the registered device list to SQLite (same store as app shutdown).
    pub(crate) fn persist_registered_devices(&self) {
        let db_path = device_store::registration_db_path();
        if let Err(e) = device_store::save_registered_devices(&db_path, &self.endpoints) {
            tracing::warn!("device_store: save {:?}: {e}", db_path);
        }
    }
}
