//! **Independent fork** of `devices/device_card.rs` for window-management VM rows.
//! Intentionally duplicated: adjust here without changing Connect device cards.
//! Many `paint_*` helpers exceed clippy’s default arity; lists stay explicit at call sites.
#![allow(clippy::too_many_arguments)]

use std::path::Path;

use egui::{
    Color32, CornerRadius, FontId, Frame, Label, Margin, Rect, RichText, Sense, TextStyle,
    TextWrapMode, Vec2, WidgetText, pos2, vec2,
};
use titan_common::VmWindowRecord;

use super::helpers_dup::{
    DEVICE_CARD_BODY_COL_GAP, DEVICE_PREVIEW_PLACEHOLDER_BG, DEVICE_PREVIEW_PLACEHOLDER_TEXT,
    card_outline, device_card_resource_values, device_card_stat_label_value_gap,
    device_card_two_col_row, device_mgmt_remark_row_interact,
};
use crate::host_app::constants::{ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, OK_GREEN, card_shadow};
use crate::host_app::model::HostApp;
use crate::titan_egui_widgets::{danger_preview_delete_button, preview_overlay_configure_button};
use crate::titan_i18n::{Msg, UiLang, host_running_windows_line, t};

const CARD_BODY_GRID_PX: f32 = 13.0;
const METRIC_BODY_ROW_GAP: f32 = 5.0;
const REMARK_ROW_H: f32 = 32.0;
const PREVIEW_HOVER_MASK_A: u8 = 100;
const PREVIEW_CFG_BTN_PAD: f32 = 8.0;
const PREVIEW_OVERLAY_BTN_H: f32 = 30.0;
const PREVIEW_OVERLAY_BTN_GAP: f32 = 8.0;

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
    vm_window_clone_card_shell(ui)
        .show(ui, |ui| {
            paint_device_masonry_frame_inner(
                app,
                ui,
                prep.row_ix,
                card_w,
                lang,
                false,
                &prep.label_s,
                &prep.addr_s,
                prep.win_n,
                false,
                &prep.preview_key,
                prep.remark.as_str(),
            )
        })
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

#[rustfmt::skip]
fn paint_device_masonry_frame_inner(app: &mut HostApp, ui: &mut egui::Ui, row_ix: usize, card_w: f32, lang: UiLang, is_sel: bool, label_s: &str, addr_s: &str, win_n: u32, online: bool, preview_key: &str, remark_body: &str) -> egui::Response {
    device_card_set_fixed_width(ui, card_w);
    let card_tl = ui.cursor().min;
    let mut select_split_y = card_tl.y;
    let mut select_interact_top_y = card_tl.y;
    ui.vertical(|ui| {
        paint_device_card_column(
            app,
            ui,
            row_ix,
            card_w,
            lang,
            is_sel,
            label_s,
            addr_s,
            win_n,
            online,
            preview_key,
            remark_body,
            &mut select_split_y,
            &mut select_interact_top_y,
        );
    });
    device_card_select_interact(ui, pos2(card_tl.x, select_interact_top_y), select_split_y, row_ix)
}

fn device_card_set_fixed_width(ui: &mut egui::Ui, card_w: f32) {
    ui.set_width(card_w);
    ui.set_min_width(card_w);
    ui.set_max_width(card_w);
}

#[rustfmt::skip]
fn paint_device_card_column(app: &mut HostApp, ui: &mut egui::Ui, row_ix: usize, card_w: f32, lang: UiLang, is_sel: bool, label_s: &str, addr_s: &str, win_n: u32, online: bool, preview_key: &str, remark_body: &str, select_split_y: &mut f32, select_interact_top_y: &mut f32) {
    ui.spacing_mut().item_spacing.y = 0.0;
    paint_device_preview_slot(app, ui, row_ix, preview_key, card_w, lang, online);
    *select_interact_top_y = ui.cursor().min.y;
    Frame::NONE.inner_margin(Margin::symmetric(12, 10)).show(ui, |ui| {
        let inner_w = (card_w - 24.0).max(1.0);
        ui.set_width(inner_w);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            paint_device_status_and_metrics(ui, lang, app, preview_key, online, is_sel, label_s, inner_w, addr_s, win_n, select_split_y);
            paint_vm_dup_remark_block(ui, lang, inner_w, remark_body);
        });
    });
}

fn device_preview_slot_height(card_w: f32) -> f32 {
    (card_w * 9.0 / 16.0).clamp(100.0, 200.0)
}

fn preview_slot_top_corners() -> CornerRadius {
    CornerRadius {
        nw: CARD_CORNER_RADIUS.nw,
        ne: CARD_CORNER_RADIUS.ne,
        sw: 0,
        se: 0,
    }
}

fn paint_device_preview_fill(
    app: &HostApp,
    ui: &mut egui::Ui,
    preview_key: &str,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    lang: UiLang,
) {
    if let Some(tex) = app.host_desktop_textures.get(preview_key) {
        paint_preview_texture(ui, preview_rect, preview_corners, tex);
    } else {
        paint_preview_placeholder(ui, preview_rect, preview_corners, lang);
    }
}

fn paint_preview_hover_mask(ui: &egui::Ui, preview_rect: Rect, preview_corners: CornerRadius) {
    ui.painter().rect_filled(
        preview_rect,
        preview_corners,
        Color32::from_black_alpha(PREVIEW_HOVER_MASK_A),
    );
}

/// Right-aligned row: **[配置] 8px [删除]** (delete flush to preview right inset).
fn preview_overlay_action_bar_rects(preview_rect: Rect) -> (Rect, Rect) {
    let pad = PREVIEW_CFG_BTN_PAD;
    let gap = PREVIEW_OVERLAY_BTN_GAP;
    let y = preview_rect.bottom() - pad - PREVIEW_OVERLAY_BTN_H;
    let max_pair = (preview_rect.width() - pad * 2.0 - gap).max(0.0);
    let w_cfg = (max_pair * 0.52).clamp(56.0, 120.0);
    let w_del = (max_pair - gap - w_cfg).clamp(48.0, 100.0);
    let right_x = preview_rect.right() - pad;
    let del_min = pos2(right_x - w_del, y);
    let cfg_min = pos2(right_x - w_del - gap - w_cfg, y);
    (
        Rect::from_min_size(cfg_min, vec2(w_cfg, PREVIEW_OVERLAY_BTN_H)),
        Rect::from_min_size(del_min, vec2(w_del, PREVIEW_OVERLAY_BTN_H)),
    )
}

fn paint_preview_delete_btn(
    ui: &mut egui::Ui,
    btn_rect: Rect,
    lang: UiLang,
    app: &mut HostApp,
    card_index: usize,
) {
    if danger_preview_delete_button(ui, btn_rect, t(lang, Msg::DeviceMgmtPreviewDelete)).clicked() {
        app.pending_remove_endpoint = Some(card_index);
        ui.ctx().request_repaint();
    }
}

fn paint_preview_configure_btn(
    ui: &mut egui::Ui,
    btn_rect: Rect,
    lang: UiLang,
    app: &mut HostApp,
    card_index: usize,
) {
    if preview_overlay_configure_button(ui, btn_rect, t(lang, Msg::DeviceMgmtPreviewConfigure))
        .clicked()
    {
        app.open_host_config_from_card(card_index);
    }
}

fn paint_device_preview_hover_layer(
    ui: &mut egui::Ui,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    lang: UiLang,
    hovered: bool,
    app: &mut HostApp,
    card_index: usize,
) {
    if !hovered {
        return;
    }
    paint_preview_hover_mask(ui, preview_rect, preview_corners);
    let (cfg_rect, del_rect) = preview_overlay_action_bar_rects(preview_rect);
    paint_preview_configure_btn(ui, cfg_rect, lang, app, card_index);
    paint_preview_delete_btn(ui, del_rect, lang, app, card_index);
}

fn paint_device_preview_slot(
    app: &mut HostApp,
    ui: &mut egui::Ui,
    card_index: usize,
    preview_key: &str,
    card_w: f32,
    lang: UiLang,
    online: bool,
) {
    let preview_h = device_preview_slot_height(card_w);
    let (preview_rect, _) = ui.allocate_exact_size(Vec2::new(card_w, preview_h), Sense::empty());
    let corners = preview_slot_top_corners();
    paint_device_preview_fill(app, ui, preview_key, preview_rect, corners, lang);
    // Use geometry + layer clip, not `Response::hovered()`: the latter goes false when the pointer
    // moves onto overlay `ui.put` buttons, which made the overlay disappear before click.
    let show_chrome = online && ui.rect_contains_pointer(preview_rect);
    paint_device_preview_hover_layer(
        ui,
        preview_rect,
        corners,
        lang,
        show_chrome,
        app,
        card_index,
    );
    if show_chrome {
        ui.ctx().request_repaint();
    }
}

fn paint_preview_texture(
    ui: &mut egui::Ui,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    tex: &egui::TextureHandle,
) {
    ui.put(
        preview_rect,
        egui::Image::new(tex)
            .corner_radius(preview_corners)
            .fit_to_exact_size(preview_rect.size())
            .maintain_aspect_ratio(false),
    );
}

fn paint_preview_placeholder(
    ui: &mut egui::Ui,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    lang: UiLang,
) {
    ui.painter()
        .rect_filled(preview_rect, preview_corners, DEVICE_PREVIEW_PLACEHOLDER_BG);
    let note = t(lang, Msg::DeviceMgmtDesktopPreviewNote);
    let galley = WidgetText::from(
        RichText::new(note)
            .small()
            .color(DEVICE_PREVIEW_PLACEHOLDER_TEXT),
    )
    .into_galley(
        ui,
        Some(TextWrapMode::Wrap),
        preview_rect.width() * 0.92,
        TextStyle::Body,
    );
    let pos = preview_rect.center() - galley.size() * 0.5;
    ui.painter()
        .galley(pos, galley, DEVICE_PREVIEW_PLACEHOLDER_TEXT);
}

fn paint_device_status_and_metrics(
    ui: &mut egui::Ui,
    lang: UiLang,
    app: &HostApp,
    preview_key: &str,
    online: bool,
    is_sel: bool,
    label_s: &str,
    inner_w: f32,
    addr_s: &str,
    win_n: u32,
    select_split_y: &mut f32,
) {
    let metrics = device_card_metrics_tuple(app, preview_key, online);
    let weak = ui.visuals().widgets.inactive.text_color();
    let title_color = device_card_title_color_for_selection(ui, is_sel);
    paint_device_status_row(ui, lang, online, weak, title_color, label_s);
    paint_device_metric_rows_from_tuple(ui, lang, inner_w, weak, addr_s, win_n, metrics);
    *select_split_y = ui.cursor().min.y;
}

type DeviceCardMetricTuple = (f32, f64, String, String, String, String);

fn device_card_metrics_tuple(
    app: &HostApp,
    preview_key: &str,
    online: bool,
) -> DeviceCardMetricTuple {
    let st_ref = online
        .then(|| app.host_resource_stats.get(preview_key))
        .flatten();
    device_card_resource_values(online, st_ref)
}

fn device_card_title_color_for_selection(ui: &egui::Ui, is_sel: bool) -> Color32 {
    if is_sel {
        ACCENT
    } else {
        ui.visuals().strong_text_color()
    }
}

fn paint_device_metric_rows_from_tuple(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    addr_s: &str,
    win_n: u32,
    m: DeviceCardMetricTuple,
) {
    let (cpu_pct, mem_pct, net_down, net_up, disk_r, disk_w) = m;
    paint_device_metric_rows(
        ui, lang, inner_w, weak, cpu_pct, mem_pct, net_down, net_up, disk_r, disk_w, addr_s, win_n,
    );
}

fn paint_device_status_row(
    ui: &mut egui::Ui,
    lang: UiLang,
    online: bool,
    weak: Color32,
    title_color: Color32,
    label_s: &str,
) {
    const CARD_STATUS_TITLE_PX: f32 = 16.0;
    let px = CARD_STATUS_TITLE_PX;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if online {
            paint_status_online_badges(ui, lang, weak, px);
        } else {
            paint_status_offline_badges(ui, lang, px);
        }
        paint_status_title_label(ui, label_s, title_color, px);
    });
}

fn paint_status_online_badges(ui: &mut egui::Ui, lang: UiLang, weak: Color32, px: f32) {
    ui.label(RichText::new("●").size(px).color(OK_GREEN));
    ui.label(
        RichText::new(t(lang, Msg::MonitorStatOnline))
            .size(px)
            .color(weak),
    );
}

fn paint_status_offline_badges(ui: &mut egui::Ui, lang: UiLang, px: f32) {
    ui.label(RichText::new("○").size(px).weak());
    ui.label(
        RichText::new(t(lang, Msg::MonitorStatOffline))
            .size(px)
            .weak(),
    );
}

fn paint_status_title_label(ui: &mut egui::Ui, label_s: &str, title_color: Color32, px: f32) {
    ui.add(Label::new(RichText::new(label_s).strong().size(px).color(title_color)).truncate());
}

fn paint_device_metric_rows_cpu_net_block(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    cpu_pct: f32,
    net_down: String,
    net_up: String,
) {
    paint_metric_row_cpu_net(
        ui,
        lang,
        inner_w,
        weak,
        CARD_BODY_GRID_PX,
        cpu_pct,
        net_down,
        net_up,
    );
    ui.add_space(METRIC_BODY_ROW_GAP);
}

fn paint_device_metric_rows_mem_disk_block(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    mem_pct: f64,
    disk_r: String,
    disk_w: String,
) {
    paint_metric_row_mem_disk(
        ui,
        lang,
        inner_w,
        weak,
        CARD_BODY_GRID_PX,
        mem_pct,
        disk_r,
        disk_w,
    );
    ui.add_space(METRIC_BODY_ROW_GAP);
}

fn paint_device_metric_rows(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    cpu_pct: f32,
    mem_pct: f64,
    net_down: String,
    net_up: String,
    disk_r: String,
    disk_w: String,
    addr_s: &str,
    win_n: u32,
) {
    paint_device_metric_rows_cpu_net_block(ui, lang, inner_w, weak, cpu_pct, net_down, net_up);
    paint_device_metric_rows_mem_disk_block(ui, lang, inner_w, weak, mem_pct, disk_r, disk_w);
    paint_metric_row_addr_win(ui, lang, inner_w, CARD_BODY_GRID_PX, addr_s, win_n);
    ui.add_space(METRIC_BODY_ROW_GAP);
}

fn metric_rich_cpu_pct(lang: UiLang, grid_px: f32, weak: Color32, cpu_pct: f32) -> RichText {
    RichText::new(format!(
        "{} {:.1}%",
        t(lang, Msg::DeviceMgmtResCpu),
        cpu_pct
    ))
    .size(grid_px)
    .color(weak)
}

fn metric_rich_net_pair(
    lang: UiLang,
    grid_px: f32,
    weak: Color32,
    net_down: &str,
    net_up: &str,
) -> RichText {
    RichText::new(format!(
        "{} {} / {}",
        t(lang, Msg::DeviceMgmtResNet),
        net_down,
        net_up
    ))
    .size(grid_px)
    .color(weak)
}

fn paint_metric_row_cpu_net(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    grid_px: f32,
    cpu_pct: f32,
    net_down: String,
    net_up: String,
) {
    device_card_two_col_row(
        ui,
        inner_w,
        DEVICE_CARD_BODY_COL_GAP,
        metric_rich_cpu_pct(lang, grid_px, weak, cpu_pct),
        metric_rich_net_pair(lang, grid_px, weak, &net_down, &net_up),
    );
}

fn metric_rich_mem_pct(lang: UiLang, grid_px: f32, weak: Color32, mem_pct: f64) -> RichText {
    RichText::new(format!(
        "{} {:.0}%",
        t(lang, Msg::DeviceMgmtResMem),
        mem_pct
    ))
    .size(grid_px)
    .color(weak)
}

fn metric_rich_disk_io(
    lang: UiLang,
    grid_px: f32,
    weak: Color32,
    disk_r: &str,
    disk_w: &str,
) -> RichText {
    RichText::new(format!(
        "{} {} / {}",
        t(lang, Msg::DeviceMgmtResDiskIo),
        disk_r,
        disk_w
    ))
    .size(grid_px)
    .color(weak)
}

fn paint_metric_row_mem_disk(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    grid_px: f32,
    mem_pct: f64,
    disk_r: String,
    disk_w: String,
) {
    device_card_two_col_row(
        ui,
        inner_w,
        DEVICE_CARD_BODY_COL_GAP,
        metric_rich_mem_pct(lang, grid_px, weak, mem_pct),
        metric_rich_disk_io(lang, grid_px, weak, &disk_r, &disk_w),
    );
}

fn paint_metric_row_addr_win(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    grid_px: f32,
    addr_s: &str,
    win_n: u32,
) {
    let addr_win_color = ui.visuals().widgets.inactive.text_color();
    device_card_two_col_row(
        ui,
        inner_w,
        DEVICE_CARD_BODY_COL_GAP,
        RichText::new(addr_s)
            .monospace()
            .size(grid_px)
            .color(addr_win_color),
        RichText::new(host_running_windows_line(lang, win_n))
            .size(grid_px)
            .color(addr_win_color),
    );
}

fn paint_vm_dup_remark_block(ui: &mut egui::Ui, lang: UiLang, inner_w: f32, rem: &str) {
    let (weak, remark_font, title_rt, stat_lbl_gap) = remark_block_style(ui, lang);
    let hint = t(lang, Msg::DeviceMgmtRemarkDblclkHint);
    let right_rt = remark_display_right_richtext(rem, hint, &remark_font, weak);
    let touch_id = egui::Id::new(("vm_window_clone_remark", rem));
    let _ = device_mgmt_remark_row_interact(
        ui,
        inner_w,
        stat_lbl_gap,
        title_rt,
        right_rt,
        touch_id,
        REMARK_ROW_H,
    );
}

fn remark_block_style(ui: &egui::Ui, lang: UiLang) -> (Color32, FontId, RichText, f32) {
    let weak = ui.visuals().widgets.inactive.text_color();
    let remark_font = FontId::proportional(CARD_BODY_GRID_PX);
    let title_rt = RichText::new(t(lang, Msg::DeviceMgmtRemarkTitle))
        .font(remark_font.clone())
        .color(weak);
    let stat_lbl_gap = device_card_stat_label_value_gap(ui, CARD_BODY_GRID_PX);
    (weak, remark_font, title_rt, stat_lbl_gap)
}

fn remark_display_right_richtext(
    rem: &str,
    hint: &'static str,
    remark_font: &FontId,
    weak: Color32,
) -> RichText {
    if rem.is_empty() {
        RichText::new(hint).font(remark_font.clone()).color(weak)
    } else {
        RichText::new(rem).font(remark_font.clone()).color(weak)
    }
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
