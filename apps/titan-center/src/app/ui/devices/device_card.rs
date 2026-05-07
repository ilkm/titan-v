//! Single device card painting (Connect tab masonry).
//! Many `paint_*` helpers exceed clippy’s default arity; lists stay explicit at call sites.
//! `#[rustfmt::skip]` on a few `paint_*` fns keeps the `fn` signature on one line so
//! `tools/check_fn_code_lines.py` (30 code-line cap incl. signature) stays satisfied.
#![allow(clippy::too_many_arguments)]

mod metrics;
mod preview;
mod remark;

use egui::{Frame, Margin, Rect, Sense, pos2};

use super::helpers::card_outline;
use crate::app::CenterApp;
use crate::app::constants::{CARD_CORNER_RADIUS, CARD_SURFACE, card_shadow};
use crate::app::i18n::UiLang;

pub(super) const CARD_BODY_GRID_PX: f32 = 13.0;
pub(super) const METRIC_BODY_ROW_GAP: f32 = 5.0;
pub(super) const REMARK_ROW_H: f32 = 32.0;

#[derive(Copy, Clone)]
struct DeviceCardMeta<'a> {
    i: usize,
    card_w: f32,
    lang: UiLang,
    is_sel: bool,
    label_s: &'a str,
    addr_s: &'a str,
    win_n: u32,
    online: bool,
}

pub(super) fn paint_device_masonry_slot(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    i: usize,
    card_w: f32,
    lang: UiLang,
) -> egui::Response {
    let is_sel = app.selected_host == i;
    let (label_s, addr_s, win_n, online) = device_card_endpoint_meta(app, i);
    let meta = DeviceCardMeta {
        i,
        card_w,
        lang,
        is_sel,
        label_s: &label_s,
        addr_s: &addr_s,
        win_n,
        online,
    };
    let stroke = card_outline(ui);
    Frame::NONE
        .fill(CARD_SURFACE)
        .stroke(stroke)
        .corner_radius(CARD_CORNER_RADIUS)
        .shadow(card_shadow())
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| paint_device_masonry_frame_inner(app, ui, meta))
        .inner
}

fn paint_device_masonry_frame_inner(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    meta: DeviceCardMeta<'_>,
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
        meta.i,
    )
}

fn device_card_set_fixed_width(ui: &mut egui::Ui, card_w: f32) {
    ui.set_width(card_w);
    ui.set_min_width(card_w);
    ui.set_max_width(card_w);
}

fn paint_device_card_column(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    meta: DeviceCardMeta<'_>,
    select_split_y: &mut f32,
    select_interact_top_y: &mut f32,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let preview_key = CenterApp::endpoint_addr_key(meta.addr_s);
    preview::paint_device_preview_slot(app, ui, meta.i, &preview_key, meta.card_w, meta.lang);
    *select_interact_top_y = ui.cursor().min.y;
    Frame::NONE
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            paint_device_card_metrics_and_remark(app, ui, meta, &preview_key, select_split_y);
        });
}

fn paint_device_card_metrics_and_remark(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    meta: DeviceCardMeta<'_>,
    preview_key: &str,
    select_split_y: &mut f32,
) {
    let inner_w = (meta.card_w - 24.0).max(1.0);
    ui.set_width(inner_w);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        metrics::paint_device_status_and_metrics(
            ui,
            meta.lang,
            app,
            preview_key,
            meta.online,
            meta.is_sel,
            meta.label_s,
            inner_w,
            meta.addr_s,
            meta.win_n,
            select_split_y,
        );
        remark::paint_device_remark_block(app, ui, meta.i, meta.lang, inner_w);
    });
}

fn device_card_endpoint_meta(app: &CenterApp, i: usize) -> (String, String, u32, bool) {
    let ep = &app.endpoints[i];
    (
        ep.label.clone(),
        ep.addr.clone(),
        ep.last_vm_count,
        ep.last_known_online,
    )
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
        ui.make_persistent_id(("device_mgmt_card", i)),
        Sense::click(),
    )
}
