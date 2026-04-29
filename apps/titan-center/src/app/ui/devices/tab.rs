//! Connect-tab device grid: masonry layout and add-host entry points.
//! Masonry stepping uses more than clippy’s default argument count without a throwaway builder.
#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use egui::{
    Align, Galley, Layout, Rect, RichText, Sense, TextStyle, TextWrapMode, UiBuilder, Vec2,
    WidgetText, pos2,
};

use super::helpers::{device_mgmt_card_height_hint, device_mgmt_cols_and_card_width};
use crate::app::CenterApp;
use crate::app::constants::DEVICE_CARD_GAP;
use crate::app::discovery;
use crate::app::i18n::{Msg, t};
use crate::app::ui::widgets::subtle_button_toolbar;

impl CenterApp {
    /// Device management: cards sit directly in the page scroll (no inner list container).
    /// Each card is **vertical**: full-bleed desktop preview on top (no inner frame), padded text block below (title → status/stats → address).
    pub(crate) fn panel_device_management(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        ui.spacing_mut().item_spacing.y = 10.0;
        self.panel_device_mgmt_toolbar(ui, lang);
        ui.add_space(12.0);
        if self.endpoints.is_empty() {
            self.panel_device_mgmt_empty_state(ui, lang);
        } else {
            self.panel_device_mgmt_masonry(ui, lang);
        }
        // After all cards paint (overlay delete may set pending mid-layout).
        self.apply_pending_endpoint_remove();
        self.show_add_host_dialog(ui, lang);
    }

    fn panel_device_mgmt_toolbar(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            self.panel_device_mgmt_toolbar_left(ui, lang);
            self.panel_device_mgmt_toolbar_right(ui, lang);
        });
    }

    fn panel_device_mgmt_toolbar_left(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        if subtle_button_toolbar(ui, t(lang, Msg::BtnAddHost), true).clicked() {
            self.add_host_dialog_ip = discovery::default_manual_host_ipv4_string();
            self.add_host_dialog_port = "7788".into();
            self.add_host_dialog_err.clear();
            self.add_host_dialog_open = true;
        }
    }

    pub(crate) fn apply_pending_endpoint_remove(&mut self) {
        if let Some(idx) = self.pending_remove_endpoint.take()
            && idx < self.endpoints.len()
        {
            self.remove_endpoint_at(idx);
            self.persist_registered_devices();
        }
    }

    /// Remove one registered endpoint by index (Connect tab: card overlay delete).
    pub(crate) fn remove_endpoint_at(&mut self, idx: usize) {
        if idx >= self.endpoints.len() {
            return;
        }
        self.device_remark_edit_index = None;
        self.device_remark_edit_focus_next = false;
        if self.host_config_window_open && idx == self.selected_host {
            self.host_config_window_open = false;
        }
        self.endpoints.remove(idx);
        let new_len = self.endpoints.len();
        if new_len == 0 {
            self.selected_host = 0;
            return;
        }
        if idx < self.selected_host {
            self.selected_host -= 1;
        } else if idx == self.selected_host {
            self.selected_host = self.selected_host.min(new_len - 1);
        }
    }

    pub(crate) fn panel_device_mgmt_toolbar_right(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        if subtle_button_toolbar(
            ui,
            t(lang, Msg::BtnHostHello),
            !self.fleet_busy && !self.endpoints.is_empty(),
        )
        .clicked()
        {
            self.spawn_fleet_hello_selected();
        }
        if subtle_button_toolbar(
            ui,
            t(lang, Msg::BtnHostTelemetry),
            !self.endpoints.is_empty(),
        )
        .clicked()
        {
            self.spawn_fleet_telemetry_selected();
        }
    }

    fn panel_device_mgmt_empty_state(&self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        let w = ui.available_width();
        let h = ui.available_height().max(180.0);
        ui.allocate_ui_with_layout(egui::vec2(w, h), Layout::top_down(Align::Min), |ui| {
            Self::paint_device_mgmt_empty_centered(ui, lang, w);
        });
    }

    fn device_mgmt_empty_main_galley(
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
        text_width: f32,
        color: egui::Color32,
    ) -> Arc<Galley> {
        WidgetText::from(
            RichText::new(t(lang, Msg::DeviceMgmtNoRegistered))
                .size(15.0)
                .color(color),
        )
        .into_galley(ui, Some(TextWrapMode::Wrap), text_width, TextStyle::Body)
    }

    fn device_mgmt_empty_hint_galley(
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
        text_width: f32,
        color: egui::Color32,
    ) -> Arc<Galley> {
        WidgetText::from(
            RichText::new(t(lang, Msg::DeviceMgmtEmptyHint))
                .small()
                .line_height(Some(20.0))
                .color(color),
        )
        .into_galley(ui, Some(TextWrapMode::Wrap), text_width, TextStyle::Small)
    }

    fn device_mgmt_empty_galleys(
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
        text_width: f32,
    ) -> (Arc<Galley>, Arc<Galley>, egui::Color32, egui::Color32) {
        let main_color = ui.visuals().widgets.inactive.text_color();
        let hint_color = ui.visuals().weak_text_color();
        let main_galley = Self::device_mgmt_empty_main_galley(ui, lang, text_width, main_color);
        let hint_galley = Self::device_mgmt_empty_hint_galley(ui, lang, text_width, hint_color);
        (main_galley, hint_galley, main_color, hint_color)
    }

    fn paint_device_mgmt_empty_centered(ui: &mut egui::Ui, lang: crate::app::i18n::UiLang, w: f32) {
        let rect = ui.max_rect();
        let text_width = (w * 0.92).clamp(1.0, 520.0);
        let (main_galley, hint_galley, main_color, hint_color) =
            Self::device_mgmt_empty_galleys(ui, lang, text_width);
        let gap = 10.0;
        let main_h = main_galley.size().y;
        let block_h = main_h + gap + hint_galley.size().y;
        let block_w = main_galley.size().x.max(hint_galley.size().x);
        let origin = rect.center() - 0.5 * Vec2::new(block_w, block_h);
        ui.painter().galley(origin, main_galley, main_color);
        ui.painter().galley(
            origin + Vec2::new(0.0, main_h + gap),
            hint_galley,
            hint_color,
        );
        let _ = ui.allocate_exact_size(rect.size(), Sense::empty());
    }

    fn device_masonry_outer_metrics(inner: f32) -> (usize, f32, f32, f32, f32) {
        let (cols, card_w) = device_mgmt_cols_and_card_width(inner);
        let gap = DEVICE_CARD_GAP;
        let row_w = cols as f32 * card_w + (cols.saturating_sub(1) as f32) * gap;
        let lead = ((inner - row_w).max(0.0)) * 0.5;
        (cols, card_w, gap, row_w, lead)
    }

    fn device_masonry_col_x_starts(cols: usize, start_x: f32, card_w: f32, gap: f32) -> Vec<f32> {
        (0..cols)
            .map(|c| start_x + c as f32 * (card_w + gap))
            .collect()
    }

    fn panel_device_mgmt_masonry(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        let inner = ui.available_width();
        let (cols, card_w, gap, _row_w, lead) = Self::device_masonry_outer_metrics(inner);
        let n = self.endpoints.len();
        const CARD_STACK_GAP: f32 = 14.0;
        self.device_masonry_prune_heights();
        let columns = self.device_masonry_build_columns(n, cols, card_w, CARD_STACK_GAP);
        let grid_tl = ui.cursor().min;
        let start_x = grid_tl.x + lead;
        let y0 = grid_tl.y;
        let mut col_y = vec![y0; cols];
        let col_x = Self::device_masonry_col_x_starts(cols, start_x, card_w, gap);
        self.device_masonry_paint_columns(
            ui,
            lang,
            &columns,
            &col_x,
            &mut col_y,
            card_w,
            CARD_STACK_GAP,
        );
        // Do not call `allocate_space` for the masonry height: each `allocate_new_ui` already
        // expands the parent `min_rect` via the placer. A second allocation duplicated scroll extent.
    }

    fn device_masonry_prune_heights(&mut self) {
        self.device_masonry_heights.retain(|k, _| {
            self.endpoints
                .iter()
                .any(|e| Self::endpoint_addr_key(&e.addr) == *k)
        });
    }

    fn device_masonry_build_columns(
        &mut self,
        n: usize,
        cols: usize,
        card_w: f32,
        stack_gap: f32,
    ) -> Vec<Vec<usize>> {
        let mut col_load: Vec<f32> = vec![0.0; cols];
        let mut columns: Vec<Vec<usize>> = (0..cols).map(|_| Vec::new()).collect();
        for i in 0..n {
            let c = (0..cols)
                .min_by(|&a, &b| col_load[a].total_cmp(&col_load[b]))
                .unwrap();
            let key = Self::endpoint_addr_key(&self.endpoints[i].addr);
            let est = self
                .device_masonry_heights
                .get(&key)
                .copied()
                .filter(|&h| h >= 8.0)
                .unwrap_or_else(|| device_mgmt_card_height_hint(card_w));
            columns[c].push(i);
            col_load[c] += est + stack_gap;
        }
        columns
    }

    fn device_masonry_paint_columns(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
        columns: &[Vec<usize>],
        col_x: &[f32],
        col_y: &mut [f32],
        card_w: f32,
        stack_gap: f32,
    ) {
        for c in 0..columns.len() {
            for &i in &columns[c] {
                let used = self
                    .device_masonry_paint_one_slot(ui, lang, i, c, col_x, col_y, card_w, stack_gap);
                col_y[c] += used + stack_gap;
            }
        }
    }

    fn device_masonry_slot_height(&mut self, i: usize, card_w: f32) -> (String, f32) {
        let addr_key = Self::endpoint_addr_key(&self.endpoints[i].addr);
        let mut slot_h = self
            .device_masonry_heights
            .get(&addr_key)
            .copied()
            .unwrap_or(0.0);
        if slot_h < 8.0 {
            slot_h = device_mgmt_card_height_hint(card_w);
        }
        (addr_key, slot_h)
    }

    fn device_masonry_paint_one_slot(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
        i: usize,
        c: usize,
        col_x: &[f32],
        col_y: &[f32],
        card_w: f32,
        _stack_gap: f32,
    ) -> f32 {
        let (addr_key, slot_h) = self.device_masonry_slot_height(i, card_w);
        let rect = Rect::from_min_size(pos2(col_x[c], col_y[c]), Vec2::new(card_w, slot_h));
        let slot = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            super::device_card::paint_device_masonry_slot(self, ui, i, card_w, lang)
        });
        if slot.inner.clicked() {
            self.select_endpoint_host(i);
        }
        let used = slot.response.rect.height().max(32.0);
        self.device_masonry_heights.insert(addr_key, used);
        used
    }
}
