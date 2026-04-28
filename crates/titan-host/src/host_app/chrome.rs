//! Top bar, left nav, central stack, and language popup (layout aligned with Titan Center).

use eframe::egui::{
    self, Align, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextStyle,
    TextWrapMode, WidgetText,
};

use titan_common::UiLang;

use crate::titan_egui_widgets::{
    inset_single_select_dropdown, show_settings_tool_window, subtle_button, InsetDropdownLayout,
};

use crate::titan_i18n::{self as i18n, Msg};

use super::constants::{
    card_shadow, ACCENT, CONTENT_MAX_WIDTH, NAV_ITEM_HEIGHT, SIDEBAR_DEFAULT_WIDTH,
};
use super::model::HostApp;

fn effective_content_width(full_w: f32) -> f32 {
    let scalable = (CONTENT_MAX_WIDTH * 1.15).max(full_w * 0.92);
    full_w.min(scalable).max(280.0)
}

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

fn host_lang_settings_body(ui: &mut egui::Ui, ui_lang: &mut UiLang, lang: UiLang) {
    let full_w = ui.available_width();
    inset_single_select_dropdown(
        ui,
        "titan_host_ui_lang",
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

impl HostApp {
    pub(crate) fn render_host_top_panel(&mut self, ctx: &egui::Context) {
        let visuals = ctx.style().visuals.clone();
        let top_fill = visuals.panel_fill;
        egui::TopBottomPanel::top("host_status")
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
                    self.host_top_status_bar(ui);
                });
            });
    }

    fn host_top_status_bar(&mut self, ui: &mut egui::Ui) {
        let lang = self.persist.ui_lang;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(
                RichText::new(i18n::t(lang, Msg::HpWinTitle))
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
            let lang_btn = lang_btn.on_hover_text(i18n::t(lang, Msg::SettingsTooltip));
            self.settings_lang_btn_rect = Some(lang_btn.rect);
            if lang_btn.clicked() {
                self.settings_open = !self.settings_open;
            }
        });
    }

    fn host_side_nav_one_row(&mut self, ui: &mut egui::Ui, lang: UiLang, tab: usize, msg: Msg) {
        let selected = self.active_tab == tab;
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
            self.active_tab = tab;
        }
    }

    fn host_side_nav_tab_buttons(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.spacing_mut().item_spacing.y = 6.0;
        self.host_side_nav_one_row(ui, lang, 0, Msg::HpTabWindowMgmt);
        self.host_side_nav_one_row(ui, lang, 1, Msg::HpTabSettings);
    }

    pub(crate) fn render_host_side_nav(&mut self, ctx: &egui::Context) {
        let lang = self.persist.ui_lang;
        let visuals = ctx.style().visuals.clone();
        let nav_fill = visuals.extreme_bg_color;
        egui::SidePanel::left("host_nav")
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
                self.host_side_nav_tab_buttons(ui, lang);
            });
    }

    fn host_central_padded(&mut self, ui: &mut egui::Ui) {
        let full_w = ui.available_width();
        let content_w = effective_content_width(full_w);
        let column_w = content_w.min(full_w);
        let side_remain = (full_w - column_w).max(0.0);
        let half = 0.5 * side_remain;
        ui.horizontal(|ui| {
            ui.add_space(half);
            ui.vertical(|ui| {
                ui.set_width(column_w);
                match self.active_tab {
                    0 => self.panel_batch(ui),
                    1 => self.panel_service(ui),
                    _ => {}
                }
            });
            ui.add_space(half);
        });
    }

    pub(crate) fn render_host_central_panel(&mut self, ctx: &egui::Context) {
        let central_fill = ctx.style().visuals.window_fill;
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(central_fill)
                    .inner_margin(Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(self.active_tab as u8)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.host_central_padded(ui);
                    });
            });
    }

    pub(crate) fn render_host_lang_settings_window(&mut self, ctx: &egui::Context) {
        let lang = self.persist.ui_lang;
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
                host_lang_settings_body(ui, &mut self.persist.ui_lang, lang);
            },
        );
    }
}
