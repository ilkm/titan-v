//! Window management (host-side, viewer): Center is the sole creator/persister; host renders rows pushed via TCP.
#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use egui::{
    Align, Galley, Layout, RichText, Sense, TextStyle, TextWrapMode, UiBuilder, Vec2, WidgetText,
    pos2,
};
use titan_common::{UiLang, VmWindowRecord};

use crate::host_app::constants::DEVICE_CARD_GAP;
use crate::host_app::model::HostApp;
use crate::host_app::vm_window_device_card_clone::paint_vm_window_device_card_clone;
use crate::host_app::vm_window_grid_metrics::{
    device_mgmt_card_height_hint, device_mgmt_cols_and_card_width,
};
use crate::titan_i18n::{Msg, t};

impl HostApp {
    pub(crate) fn panel_window_mgmt(&mut self, ui: &mut egui::Ui) {
        let lang = self.persist.ui_lang;
        ui.spacing_mut().item_spacing.y = 10.0;
        if self.vm_window_records.is_empty() {
            self.panel_window_mgmt_empty_state(ui, lang);
        } else {
            self.panel_window_mgmt_masonry(ui, lang);
        }
    }

    fn panel_window_mgmt_empty_state(&self, ui: &mut egui::Ui, lang: UiLang) {
        let w = ui.available_width();
        let h = ui.available_height().max(180.0);
        ui.allocate_ui_with_layout(egui::vec2(w, h), Layout::top_down(Align::Min), |ui| {
            Self::paint_window_mgmt_empty_left(ui, lang, w);
        });
    }

    fn window_mgmt_empty_main_galley(
        ui: &mut egui::Ui,
        lang: UiLang,
        text_width: f32,
        color: egui::Color32,
    ) -> Arc<Galley> {
        WidgetText::from(
            RichText::new(t(lang, Msg::WinMgmtNoWindows))
                .size(15.0)
                .color(color),
        )
        .into_galley(ui, Some(TextWrapMode::Wrap), text_width, TextStyle::Body)
    }

    fn window_mgmt_empty_hint_galley(
        ui: &mut egui::Ui,
        lang: UiLang,
        text_width: f32,
        color: egui::Color32,
    ) -> Arc<Galley> {
        WidgetText::from(
            RichText::new(t(lang, Msg::WinMgmtEmptyHint))
                .small()
                .line_height(Some(20.0))
                .color(color),
        )
        .into_galley(ui, Some(TextWrapMode::Wrap), text_width, TextStyle::Small)
    }

    fn paint_window_mgmt_empty_left(ui: &mut egui::Ui, lang: UiLang, w: f32) {
        const PAD_X: f32 = 4.0;
        const PAD_Y: f32 = 8.0;
        let rect = ui.max_rect();
        let text_width = (w - PAD_X * 2.0).clamp(1.0, 520.0);
        let main_color = ui.visuals().widgets.inactive.text_color();
        let hint_color = ui.visuals().weak_text_color();
        let main_galley = Self::window_mgmt_empty_main_galley(ui, lang, text_width, main_color);
        let hint_galley = Self::window_mgmt_empty_hint_galley(ui, lang, text_width, hint_color);
        let gap = 10.0;
        let main_h = main_galley.size().y;
        let origin = rect.min + Vec2::new(PAD_X, PAD_Y);
        ui.painter().galley(origin, main_galley, main_color);
        let hint_origin = origin + Vec2::new(0.0, main_h + gap);
        ui.painter().galley(hint_origin, hint_galley, hint_color);
        let _ = ui.allocate_exact_size(rect.size(), Sense::empty());
    }

    fn window_masonry_outer_metrics(inner: f32) -> (usize, f32, f32, f32, f32) {
        let (cols, card_w) = device_mgmt_cols_and_card_width(inner);
        let gap = DEVICE_CARD_GAP;
        let row_w = cols as f32 * card_w + (cols.saturating_sub(1) as f32) * gap;
        let lead = 0.0;
        (cols, card_w, gap, row_w, lead)
    }

    fn window_masonry_col_x_starts(cols: usize, start_x: f32, card_w: f32, gap: f32) -> Vec<f32> {
        (0..cols)
            .map(|c| start_x + c as f32 * (card_w + gap))
            .collect()
    }

    fn panel_window_mgmt_masonry(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        let rows: Vec<VmWindowRecord> = self.vm_window_records.clone();
        let inner = ui.available_width();
        let (cols, card_w, gap, _row_w, lead) = Self::window_masonry_outer_metrics(inner);
        const STACK: f32 = 14.0;
        self.window_masonry_prune_heights(&rows);
        let columns = self.window_masonry_build_columns(&rows, cols, card_w, STACK);
        let grid_tl = ui.cursor().min;
        let start_x = grid_tl.x + lead;
        let y0 = grid_tl.y;
        let mut col_y = vec![y0; cols];
        let col_x = Self::window_masonry_col_x_starts(cols, start_x, card_w, gap);
        self.window_masonry_paint_columns(
            ui, lang, &rows, &columns, &col_x, &mut col_y, card_w, STACK,
        );
    }

    fn window_masonry_prune_heights(&mut self, rows: &[VmWindowRecord]) {
        self.vm_window_masonry_heights
            .retain(|k, _| rows.iter().any(|r| &r.record_id == k));
    }

    fn window_masonry_build_columns(
        &self,
        rows: &[VmWindowRecord],
        cols: usize,
        card_w: f32,
        stack_gap: f32,
    ) -> Vec<Vec<usize>> {
        let mut col_load: Vec<f32> = vec![0.0; cols];
        let mut columns: Vec<Vec<usize>> = (0..cols).map(|_| Vec::new()).collect();
        for (i, row) in rows.iter().enumerate() {
            let c = (0..cols)
                .min_by(|&a, &b| col_load[a].total_cmp(&col_load[b]))
                .unwrap();
            let key = row.record_id.clone();
            let est = self
                .vm_window_masonry_heights
                .get(&key)
                .copied()
                .filter(|&h| h >= 8.0)
                .unwrap_or_else(|| device_mgmt_card_height_hint(card_w));
            columns[c].push(i);
            col_load[c] += est + stack_gap;
        }
        columns
    }

    fn window_masonry_paint_columns(
        &mut self,
        ui: &mut egui::Ui,
        lang: UiLang,
        rows: &[VmWindowRecord],
        columns: &[Vec<usize>],
        col_x: &[f32],
        col_y: &mut [f32],
        card_w: f32,
        stack_gap: f32,
    ) {
        for c in 0..columns.len() {
            for &i in &columns[c] {
                let used = self.window_masonry_paint_one_slot(
                    ui, lang, rows, i, c, col_x, col_y, card_w, stack_gap,
                );
                col_y[c] += used + stack_gap;
            }
        }
    }

    fn window_masonry_slot_height(
        &self,
        rows: &[VmWindowRecord],
        i: usize,
        card_w: f32,
    ) -> (String, f32) {
        let key = rows[i].record_id.clone();
        let mut slot_h = self
            .vm_window_masonry_heights
            .get(&key)
            .copied()
            .unwrap_or(0.0);
        if slot_h < 8.0 {
            slot_h = device_mgmt_card_height_hint(card_w);
        }
        (key, slot_h)
    }

    fn window_masonry_paint_one_slot(
        &mut self,
        ui: &mut egui::Ui,
        lang: UiLang,
        rows: &[VmWindowRecord],
        i: usize,
        c: usize,
        col_x: &[f32],
        col_y: &[f32],
        card_w: f32,
        _stack_gap: f32,
    ) -> f32 {
        let (key, slot_h) = self.window_masonry_slot_height(rows, i, card_w);
        let rect = egui::Rect::from_min_size(pos2(col_x[c], col_y[c]), Vec2::new(card_w, slot_h));
        let slot = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let _ = paint_vm_window_device_card_clone(self, ui, &rows[i], i, card_w, lang);
        });
        let used = slot.response.rect.height().max(32.0);
        self.vm_window_masonry_heights.insert(key, used);
        used
    }
}
