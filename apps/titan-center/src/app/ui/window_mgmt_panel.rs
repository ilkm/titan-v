//! Window management tab: same shell as Connect device management (toolbar, empty state, masonry).
#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use egui::{
    Align, Galley, Layout, Rect, RichText, Sense, TextStyle, TextWrapMode, UiBuilder, Vec2,
    WidgetText, pos2,
};
use titan_common::VmWindowRecord;

use super::devices::{device_mgmt_card_height_hint, device_mgmt_cols_and_card_width};
use super::vm_window_device_card_clone::paint_vm_window_device_card_clone;
use super::widgets::subtle_button_toolbar;
use crate::app::CenterApp;
use crate::app::constants::DEVICE_CARD_GAP;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::net::NetUiMsg;
use crate::app::vm_window_db;

impl CenterApp {
    pub(crate) fn panel_window_management(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        ui.spacing_mut().item_spacing.y = 10.0;
        self.panel_window_mgmt_toolbar(ui, lang);
        ui.add_space(12.0);
        if self.vm_window_records.is_empty() {
            self.panel_window_mgmt_empty_state(ui, lang);
        } else {
            self.panel_window_mgmt_masonry(ui, lang);
        }
        self.apply_pending_vm_window_delete();
    }

    fn apply_pending_vm_window_delete(&mut self) {
        let Some(idx) = self.pending_delete_vm_window_row_ix.take() else {
            return;
        };
        let Some(row) = self.vm_window_records.get(idx).cloned() else {
            return;
        };
        spawn_vm_window_delete_task(self.net_tx.clone(), row.record_id, row.device_id);
    }

    /// Persist the in-memory remark edit for `record_id` to SQLite, then push the refreshed
    /// snapshot to the affected host. Called from the VM-window card when the inline `TextEdit`
    /// loses focus.
    pub(crate) fn commit_vm_window_remark_edit(&mut self, record_id: &str) {
        self.vm_window_remark_edit_record_id = None;
        self.vm_window_remark_edit_focus_next = false;
        let Some(row) = self
            .vm_window_records
            .iter()
            .find(|r| r.record_id == record_id)
            .cloned()
        else {
            return;
        };
        spawn_vm_window_remark_save_task(self.net_tx.clone(), row);
    }

    fn panel_window_mgmt_toolbar(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            self.panel_window_mgmt_toolbar_left(ui, lang);
            self.panel_device_mgmt_toolbar_right(ui, lang);
        });
    }

    fn panel_window_mgmt_toolbar_left(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        if subtle_button_toolbar(ui, t(lang, Msg::WinMgmtReloadDb), true).clicked() {
            self.refetch_vm_windows_from_all_hosts();
        }
        if subtle_button_toolbar(ui, t(lang, Msg::HpWinMgmtCreateBtn), true).clicked() {
            self.open_vm_window_create_dialog();
        }
    }

    fn refetch_vm_windows_from_all_hosts(&mut self) {
        spawn_vm_window_reload_task(self.net_tx.clone());
    }

    fn panel_window_mgmt_empty_state(&self, ui: &mut egui::Ui, lang: UiLang) {
        let w = ui.available_width();
        let h = ui.available_height().max(180.0);
        ui.allocate_ui_with_layout(egui::vec2(w, h), Layout::top_down(Align::Min), |ui| {
            Self::paint_window_mgmt_empty_centered(ui, lang, w);
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

    fn paint_window_mgmt_empty_centered(ui: &mut egui::Ui, lang: UiLang, w: f32) {
        let rect = ui.max_rect();
        let text_width = (w * 0.92).clamp(1.0, 520.0);
        let main_color = ui.visuals().widgets.inactive.text_color();
        let hint_color = ui.visuals().weak_text_color();
        let main_galley = Self::window_mgmt_empty_main_galley(ui, lang, text_width, main_color);
        let hint_galley = Self::window_mgmt_empty_hint_galley(ui, lang, text_width, hint_color);
        let gap = 10.0;
        let main_h = main_galley.size().y;
        let block_h = main_h + gap + hint_galley.size().y;
        let block_w = main_galley.size().x.max(hint_galley.size().x);
        let origin = rect.center() - 0.5 * Vec2::new(block_w, block_h);
        ui.painter().galley(origin, main_galley, main_color);
        let hint_origin = origin + Vec2::new(0.0, main_h + gap);
        ui.painter().galley(hint_origin, hint_galley, hint_color);
        let _ = ui.allocate_exact_size(rect.size(), Sense::empty());
    }

    fn window_masonry_outer_metrics(inner: f32) -> (usize, f32, f32, f32, f32) {
        let (cols, card_w) = device_mgmt_cols_and_card_width(inner);
        let gap = DEVICE_CARD_GAP;
        let row_w = cols as f32 * card_w + (cols.saturating_sub(1) as f32) * gap;
        let lead = ((inner - row_w).max(0.0)) * 0.5;
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
        let rect = Rect::from_min_size(pos2(col_x[c], col_y[c]), Vec2::new(card_w, slot_h));
        let slot = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let _ = paint_vm_window_device_card_clone(self, ui, &rows[i], i, card_w, lang);
        });
        let used = slot.response.rect.height().max(32.0);
        self.vm_window_masonry_heights.insert(key, used);
        used
    }
}

fn spawn_vm_window_reload_task(tx: std::sync::mpsc::SyncSender<NetUiMsg>) {
    let _ = std::thread::Builder::new()
        .name("titan-center-vm-window-reload".into())
        .spawn(move || {
            let path = vm_window_db::center_vm_window_db_path();
            let msg = match vm_window_db::list_all(&path) {
                Ok(rows) => NetUiMsg::VmWindowReloadDone {
                    rows: Some(rows),
                    detail: String::new(),
                },
                Err(e) => NetUiMsg::VmWindowReloadDone {
                    rows: None,
                    detail: format!("vm_window_db: list_all (reload): {e}"),
                },
            };
            let _ = tx.send(msg);
        });
}

fn spawn_vm_window_delete_task(
    tx: std::sync::mpsc::SyncSender<NetUiMsg>,
    record_id: String,
    device_id: String,
) {
    let _ = std::thread::Builder::new()
        .name("titan-center-vm-window-delete".into())
        .spawn(move || {
            let path = vm_window_db::center_vm_window_db_path();
            let detail = match vm_window_db::delete_by_record_id(&path, &record_id) {
                Ok(_) => String::new(),
                Err(e) => format!("vm_window_db: delete {record_id}: {e}"),
            };
            let _ = tx.send(NetUiMsg::VmWindowDeleteDone {
                record_id,
                device_id,
                detail,
            });
        });
}

fn spawn_vm_window_remark_save_task(
    tx: std::sync::mpsc::SyncSender<NetUiMsg>,
    row: VmWindowRecord,
) {
    let _ = std::thread::Builder::new()
        .name("titan-center-vm-window-remark-save".into())
        .spawn(move || {
            let path = vm_window_db::center_vm_window_db_path();
            let detail = match vm_window_db::upsert(&path, &row) {
                Ok(()) => String::new(),
                Err(e) => format!("vm_window_db: remark upsert {}: {e}", row.record_id),
            };
            let _ = tx.send(NetUiMsg::VmWindowRemarkSaveDone {
                record_id: row.record_id,
                device_id: row.device_id,
                detail,
            });
        });
}
