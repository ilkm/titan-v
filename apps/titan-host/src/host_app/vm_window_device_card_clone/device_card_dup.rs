//! **Independent fork** of `devices/device_card.rs` for window-management VM rows.
//! Intentionally duplicated: adjust here without changing Connect device cards.
#![allow(clippy::too_many_arguments)]

mod metrics_dup;
mod preview_dup;
mod remark_dup;

use std::path::Path;

use egui::{Frame, Margin, Rect, Sense, pos2};
use titan_common::VmWindowRecord;

use super::helpers_dup::card_outline;
use crate::host_app::constants::{CARD_CORNER_RADIUS, CARD_SURFACE, card_shadow};
use crate::host_app::model::HostApp;
use crate::titan_i18n::UiLang;

pub(super) const CARD_BODY_GRID_PX: f32 = 13.0;
pub(super) const METRIC_BODY_ROW_GAP: f32 = 5.0;
pub(super) const REMARK_ROW_H: f32 = 32.0;

#[derive(Copy, Clone)]
struct VmCardMeta<'a> {
    row_ix: usize,
    card_w: f32,
    lang: UiLang,
    is_sel: bool,
    label_s: &'a str,
    addr_s: &'a str,
    win_n: u32,
    online: bool,
    preview_key: &'a str,
    record_id: &'a str,
    remark_body: &'a str,
}

struct VmWindowCardPaintPrep {
    row_ix: usize,
    label_s: String,
    addr_s: String,
    win_n: u32,
    preview_key: String,
    remark: String,
}

impl VmWindowCardPaintPrep {
    fn from_row(row: &VmWindowRecord, row_ix: usize) -> Self {
        let (label_s, addr_s, win_n) = vm_window_clone_row_meta(row);
        Self {
            row_ix,
            label_s,
            addr_s,
            win_n,
            preview_key: format!("vmwin:{}", row.record_id),
            remark: row.remark.clone(),
        }
    }
}

fn vm_window_clone_card_shell(ui: &egui::Ui) -> Frame {
    Frame::NONE
        .fill(CARD_SURFACE)
        .stroke(card_outline(ui))
        .corner_radius(CARD_CORNER_RADIUS)
        .shadow(card_shadow())
        .inner_margin(Margin::ZERO)
}

#[must_use]
pub fn paint_vm_window_device_card_clone(
    app: &mut HostApp,
    ui: &mut egui::Ui,
    row: &VmWindowRecord,
    row_ix: usize,
    card_w: f32,
    lang: UiLang,
) -> egui::Response {
    let prep = VmWindowCardPaintPrep::from_row(row, row_ix);
    let meta = VmCardMeta {
        row_ix: prep.row_ix,
        card_w,
        lang,
        is_sel: false,
        label_s: &prep.label_s,
        addr_s: &prep.addr_s,
        win_n: prep.win_n,
        online: true,
        preview_key: &prep.preview_key,
        record_id: row.record_id.as_str(),
        remark_body: prep.remark.as_str(),
    };
    vm_window_clone_card_shell(ui)
        .show(ui, |ui| paint_device_masonry_frame_inner(app, ui, meta))
        .inner
}

fn vm_window_clone_row_meta(row: &VmWindowRecord) -> (String, String, u32) {
    let title = Path::new(&row.vm_directory)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| row.record_id.chars().take(12).collect());
    (title, row.host_control_addr.clone(), 0)
}

fn paint_device_masonry_frame_inner(
    app: &mut HostApp,
    ui: &mut egui::Ui,
    meta: VmCardMeta<'_>,
) -> egui::Response {
    device_card_set_fixed_width(ui, meta.card_w);
    let card_tl = ui.cursor().min;
    let mut select_split_y = card_tl.y;
    let mut select_interact_top_y = card_tl.y;
    ui.vertical(|ui| {
        paint_device_card_column(
            app,
            ui,
            meta,
            &mut select_split_y,
            &mut select_interact_top_y,
        );
    });
    device_card_select_interact(
        ui,
        pos2(card_tl.x, select_interact_top_y),
        select_split_y,
        meta.row_ix,
    )
}

fn device_card_set_fixed_width(ui: &mut egui::Ui, card_w: f32) {
    ui.set_width(card_w);
    ui.set_min_width(card_w);
    ui.set_max_width(card_w);
}

fn paint_device_card_column(
    app: &mut HostApp,
    ui: &mut egui::Ui,
    meta: VmCardMeta<'_>,
    select_split_y: &mut f32,
    select_interact_top_y: &mut f32,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    preview_dup::paint_device_preview_slot(
        app,
        ui,
        meta.row_ix,
        meta.preview_key,
        meta.card_w,
        meta.lang,
        meta.online,
    );
    *select_interact_top_y = ui.cursor().min.y;
    Frame::NONE
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            paint_host_vm_metrics_and_remark(app, ui, meta, select_split_y);
        });
}

fn paint_host_vm_metrics_and_remark(
    app: &mut HostApp,
    ui: &mut egui::Ui,
    meta: VmCardMeta<'_>,
    select_split_y: &mut f32,
) {
    let inner_w = (meta.card_w - 24.0).max(1.0);
    ui.set_width(inner_w);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        paint_host_vm_metrics(ui, app, meta, inner_w, select_split_y);
        remark_dup::paint_vm_dup_remark_block(
            ui,
            meta.lang,
            inner_w,
            meta.record_id,
            meta.remark_body,
        );
    });
}

fn paint_host_vm_metrics(
    ui: &mut egui::Ui,
    app: &mut HostApp,
    meta: VmCardMeta<'_>,
    inner_w: f32,
    select_split_y: &mut f32,
) {
    metrics_dup::paint_device_status_and_metrics(
        ui,
        meta.lang,
        app,
        meta.preview_key,
        meta.online,
        meta.is_sel,
        meta.label_s,
        inner_w,
        meta.addr_s,
        meta.win_n,
        select_split_y,
    );
}

fn device_card_select_interact(
    ui: &mut egui::Ui,
    select_min: egui::Pos2,
    select_split_y: f32,
    i: usize,
) -> egui::Response {
    let card_br = ui.min_rect().max;
    let y1 = select_split_y.max(select_min.y + 1.0);
    let select_rect = Rect::from_min_max(select_min, pos2(card_br.x, y1));
    ui.interact(
        select_rect,
        ui.make_persistent_id(("vm_window_clone_card", i)),
        Sense::click(),
    )
}
