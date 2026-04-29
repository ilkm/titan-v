//! Main window layout: top bar, side nav, central stack, settings popup, device persistence hook.

use egui::{
    Align, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextStyle, TextWrapMode,
    WidgetText,
};

use crate::app::CenterApp;
use crate::app::constants::{
    ACCENT, CONTENT_MAX_WIDTH, NAV_ITEM_HEIGHT, SIDEBAR_DEFAULT_WIDTH, card_shadow,
};
use crate::app::device_store;
use crate::app::i18n::{self, Msg, UiLang};
use crate::app::persist_data::NavTab;
use crate::app::ui::widgets::{
    InsetDropdownLayout, inset_single_select_dropdown, show_settings_tool_window,
};

fn effective_content_width(full_w: f32) -> f32 {
    let scalable = (CONTENT_MAX_WIDTH * 1.15).max(full_w * 0.92);
    full_w.min(scalable).max(280.0)
}

/// Settings popup: outer **right-top** corner, flush right with the language button, just below it.
fn settings_popup_right_top_under_btn(btn: egui::Rect) -> egui::Pos2 {
    const GAP_Y: f32 = 6.0;
    btn.right_bottom() + egui::vec2(0.0, GAP_Y)
}

fn lang_label_for_combo_choice(ui_lang: UiLang, lang: UiLang) -> &'static str {
    match ui_lang {
        UiLang::En => i18n::t(lang, Msg::LangRadioEn),
        UiLang::Zh => i18n::t(lang, Msg::LangRadioZh),
    }
}

fn settings_window_contents(ui: &mut egui::Ui, ui_lang: &mut UiLang, lang: UiLang) {
    let full_w = ui.available_width();
    inset_single_select_dropdown(
        ui,
        "titan_center_ui_lang",
        full_w,
        lang_label_for_combo_choice(*ui_lang, lang),
        72.0,
        InsetDropdownLayout::compact(),
        |ui| {
            ui.selectable_value(ui_lang, UiLang::En, i18n::t(lang, Msg::LangRadioEn));
            ui.selectable_value(ui_lang, UiLang::Zh, i18n::t(lang, Msg::LangRadioZh));
        },
    );
}

impl CenterApp {
    pub(crate) fn render_top_panel(&mut self, ctx: &egui::Context) {
        let title = i18n::t(self.ui_lang, Msg::BrandTitle).to_string();
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Title(title));
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

    fn central_panel_horizontal_padded(&mut self, ui: &mut egui::Ui) {
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
                    NavTab::Legacy => self.panel_device_management_redirect(ui),
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
                        self.central_panel_horizontal_padded(ui);
                    });
            });
    }

    pub(crate) fn render_settings_window(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let anchor = self
            .settings_lang_btn_rect
            .map(settings_popup_right_top_under_btn);
        show_settings_tool_window(
            ctx,
            &mut self.settings_open,
            i18n::t(lang, Msg::SettingsLangWindowTitle),
            anchor,
            egui::vec2(-256.0, 48.0),
            egui::vec2(208.0, 84.0),
            |ui| {
                settings_window_contents(ui, &mut self.ui_lang, lang);
            },
        );
    }

    pub(crate) fn center_app_tick_frame(&mut self, ctx: &egui::Context) {
        self.maybe_flush_center_sqlite(ctx);
        self.tick_sync_ui_lang_to_hosts_if_needed();
        self.drain_net_inbox();
        self.tick_add_host_verify_watchdog(ctx);
        self.tick_telemetry_staleness();
        self.tick_reachability_probes(ctx);
        self.tick_auto_control_session(ctx);
        self.tick_discovery_thread();
        self.tick_host_collect_thread();
        self.tick_desktop_preview_refresh(ctx);
        self.tick_list_vms_auto_refresh(ctx);
    }

    pub(crate) fn center_app_paint_frame(&mut self, ctx: &egui::Context) {
        self.render_top_panel(ctx);
        self.render_side_nav(ctx);
        self.render_central_panel(ctx);
        self.render_settings_window(ctx);
        self.render_host_config_window(ctx);
        self.render_ui_toast(ctx);
    }

    /// Persist the registered device list to SQLite (same store as app shutdown).
    pub(crate) fn persist_registered_devices(&self) {
        let db_path = device_store::registration_db_path();
        if let Err(e) = device_store::save_registered_devices(&db_path, &self.endpoints) {
            tracing::warn!("device_store: save {:?}: {e}", db_path);
        }
    }
}
