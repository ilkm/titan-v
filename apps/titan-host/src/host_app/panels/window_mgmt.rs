//! Window management: same page shell as Titan Center (toolbar, empty state, masonry) + create modal.
#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use egui::{
    Align, Galley, Layout, RichText, Sense, TextStyle, TextWrapMode, UiBuilder, Vec2, WidgetText,
    pos2,
};
use titan_common::{UiLang, VmWindowRecord, VmWindowRegisterBeacon};

use crate::host_app::constants::DEVICE_CARD_GAP;
use crate::host_app::model::{HostApp, default_vm_directory};
use crate::host_app::vm_window_device_card_clone::paint_vm_window_device_card_clone;
use crate::host_app::vm_window_grid_metrics::{
    device_mgmt_card_height_hint, device_mgmt_cols_and_card_width,
};
use crate::titan_egui_widgets::{
    OpaqueFrameSource, form_field_row, primary_button_large, show_opaque_modal, subtle_button,
    subtle_button_toolbar,
};
use crate::titan_i18n::{self as i18n, Msg, t};
use crate::ui_persist::HostUiPersist;

const CREATE_DLG_INNER: egui::Vec2 = egui::Vec2::new(480.0, 360.0);

fn unix_millis_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn host_notify_control_addr(p: &HostUiPersist) -> String {
    let o = p.public_addr_override.trim();
    if !o.is_empty() {
        return o.to_string();
    }
    p.listen.trim().to_string()
}

fn host_notify_label(p: &HostUiPersist) -> String {
    let o = p.label_override.trim();
    if !o.is_empty() {
        return o.to_string();
    }
    whoami::fallible::hostname().unwrap_or_else(|_| "host".into())
}

impl HostApp {
    pub(crate) fn panel_window_mgmt(&mut self, ui: &mut egui::Ui) {
        let lang = self.persist.ui_lang;
        ui.spacing_mut().item_spacing.y = 10.0;
        self.panel_window_mgmt_toolbar(ui, lang);
        ui.add_space(12.0);
        if self.vm_window_records.is_empty() {
            self.panel_window_mgmt_empty_state(ui, lang);
        } else {
            self.panel_window_mgmt_masonry(ui, lang);
        }
        self.render_create_window_modal(ui.ctx(), lang);
    }

    fn panel_window_mgmt_toolbar(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            self.panel_window_mgmt_toolbar_left(ui, lang);
            self.panel_window_mgmt_toolbar_right(ui, lang);
        });
    }

    fn panel_window_mgmt_toolbar_left(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        if subtle_button_toolbar(ui, t(lang, Msg::WinMgmtReloadDb), true).clicked() {
            self.vm_window_records = crate::vm_window_local::load_vm_windows();
        }
        if subtle_button_toolbar(ui, t(lang, Msg::HpWinMgmtCreateBtn), true).clicked() {
            self.create_window.inline_err.clear();
            self.create_window.dialog_open = true;
        }
    }

    fn panel_window_mgmt_toolbar_right(&self, ui: &mut egui::Ui, lang: UiLang) {
        let _ = subtle_button_toolbar(ui, t(lang, Msg::BtnHostHello), false);
        let _ = subtle_button_toolbar(ui, t(lang, Msg::BtnHostTelemetry), false);
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
        let rect = egui::Rect::from_min_size(pos2(col_x[c], col_y[c]), Vec2::new(card_w, slot_h));
        let slot = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let _ = paint_vm_window_device_card_clone(self, ui, &rows[i], i, card_w, lang);
        });
        let used = slot.response.rect.height().max(32.0);
        self.vm_window_masonry_heights.insert(key, used);
        used
    }

    fn render_create_window_modal(&mut self, ctx: &egui::Context, lang: UiLang) {
        let mut open = self.create_window.dialog_open;
        show_opaque_modal(
            ctx,
            egui::Id::new("titan_host_create_window_modal"),
            i18n::t(lang, Msg::HpWinMgmtDialogTitle),
            &mut open,
            CREATE_DLG_INNER,
            OpaqueFrameSource::Ctx(ctx),
            |ui| {
                self.create_window_modal_body(ui, lang);
            },
        );
        self.create_window.dialog_open = open;
    }

    fn create_window_modal_body(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        if !self.create_window.inline_err.is_empty() {
            let err = ui.visuals().error_fg_color;
            ui.label(
                RichText::new(&self.create_window.inline_err)
                    .small()
                    .color(err),
            );
            ui.add_space(6.0);
        }
        self.create_window_row_cpu(ui, lang);
        self.create_window_row_mem(ui, lang);
        self.create_window_row_disk(ui, lang);
        self.create_window_modal_vm_dir_row(ui, lang);
        ui.add_space(12.0);
        self.create_window_modal_footer(ui, lang);
    }

    fn create_window_row_cpu(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpWinMgmtCpu)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.create_window.cpu_count)
                        .speed(0.25)
                        .range(1..=256),
                );
            },
        );
    }

    fn create_window_row_mem(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpWinMgmtMem)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.create_window.memory_mib)
                        .speed(64.0)
                        .range(256..=1_048_576),
                );
            },
        );
    }

    fn create_window_row_disk(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpWinMgmtDisk)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.create_window.disk_mib)
                        .speed(1024.0)
                        .range(1024..=16_777_216),
                );
            },
        );
    }

    fn create_window_modal_vm_dir_row(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        form_field_row(
            ui,
            RichText::new(i18n::t(lang, Msg::HpWinMgmtVmDir)).small(),
            |ui| {
                let w = ui.available_width().max(200.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.create_window.vm_directory)
                        .desired_width(w)
                        .hint_text(i18n::t(lang, Msg::HpWinMgmtVmDirHint)),
                );
            },
        );
    }

    fn create_window_modal_footer(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button_large(ui, i18n::t(lang, Msg::HpWinMgmtConfirm), true).clicked() {
                    self.try_submit_create_window(lang);
                }
                if subtle_button(ui, i18n::t(lang, Msg::BtnCancel), true).clicked() {
                    self.create_window.inline_err.clear();
                    self.create_window.dialog_open = false;
                }
            });
        });
    }

    fn try_submit_create_window(&mut self, lang: UiLang) {
        let dir_owned = self.create_window.vm_directory.trim().to_string();
        if dir_owned.is_empty() {
            self.create_window.inline_err = i18n::t(lang, Msg::HpWinMgmtErrDir).to_string();
            return;
        }
        self.create_window.inline_err.clear();
        if !self.record_vm_window_and_notify_center(lang, &dir_owned) {
            return;
        }
        tracing::info!(
            cpus = self.create_window.cpu_count,
            memory_mib = self.create_window.memory_mib,
            disk_mib = self.create_window.disk_mib,
            vm_dir = %dir_owned,
            "create window form submitted"
        );
        self.create_window.dialog_open = false;
        self.create_window.vm_directory = default_vm_directory();
    }

    fn build_vm_window_record(&self, vm_dir: &str) -> VmWindowRecord {
        VmWindowRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            device_id: crate::host_device_id::host_device_id_string(),
            host_control_addr: host_notify_control_addr(&self.persist),
            host_label: host_notify_label(&self.persist),
            cpu_count: self.create_window.cpu_count,
            memory_mib: self.create_window.memory_mib,
            disk_mib: self.create_window.disk_mib,
            vm_directory: vm_dir.to_string(),
            created_at_unix_ms: unix_millis_now(),
        }
    }

    fn record_vm_window_and_notify_center(&mut self, lang: UiLang, vm_dir: &str) -> bool {
        let record = self.build_vm_window_record(vm_dir);
        self.vm_window_records.push(record.clone());
        if let Err(e) = crate::vm_window_local::save_vm_windows(&self.vm_window_records) {
            tracing::warn!(error = %e, "vm_window local save");
            self.vm_window_records.pop();
            self.window_mgmt_feedback = i18n::t(lang, Msg::HpWinMgmtSaveErr).to_string();
            return false;
        }
        let reg_port = self.persist.center_register_udp_port.max(1);
        let beacon = VmWindowRegisterBeacon::new(record);
        crate::vm_window_notify::spawn_vm_window_register_beacon(beacon, reg_port);
        self.window_mgmt_feedback.clear();
        true
    }
}
