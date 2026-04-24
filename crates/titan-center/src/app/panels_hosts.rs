//! Standalone device-management cards (Connect tab); host grid was removed from Settings.

use egui::{
    pos2, Align, Align2, Color32, CornerRadius, FontId, Frame, Label, Layout, Margin, Rect,
    RichText, Sense, Stroke, TextStyle, TextWrapMode, UiBuilder, Vec2, WidgetText,
};
use std::cell::Cell;
use std::net::Ipv4Addr;
use std::str::FromStr;

use super::constants::{
    card_shadow, ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, DEVICE_CARD_GAP, DEVICE_CARD_MAX_WIDTH,
    DEVICE_CARD_MIN_WIDTH, OK_GREEN,
};
use super::discovery;
use super::i18n::{host_running_windows_line, t, Msg};
use super::widgets::{
    dialog_underline_text_row, dialog_underline_text_row_gap, opaque_dialog_frame,
    primary_button_large, subtle_button, subtle_button_large,
};
use super::CenterApp;

/// Horizontal gap between the two text columns on device cards (CPU/mem, disk, remark…).
const DEVICE_CARD_BODY_COL_GAP: f32 = 10.0;

/// Desktop preview slot when no JPEG has been received yet (unified look across cards).
const DEVICE_PREVIEW_PLACEHOLDER_BG: Color32 = Color32::from_rgb(236, 238, 242);
const DEVICE_PREVIEW_PLACEHOLDER_TEXT: Color32 = Color32::BLACK;

/// Add-host modal typography on white (light theme).
const ADD_HOST_DLG_BODY: Color32 = Color32::from_rgb(15, 23, 42);
const ADD_HOST_DLG_MUTED: Color32 = Color32::from_rgb(71, 85, 105);
const ADD_HOST_DLG_LABEL: Color32 = Color32::from_rgb(51, 65, 85);
const ADD_HOST_ERR_BG: Color32 = Color32::from_rgb(255, 241, 242);
const ADD_HOST_ERR_BORDER: Color32 = Color32::from_rgb(254, 202, 202);
const ADD_HOST_ERR_TEXT: Color32 = Color32::from_rgb(185, 28, 28);

/// Compact rates for narrow device cards (K/M suffix, no `/s` — keeps columns narrow).
fn format_rate_bps_short(bps: u64) -> String {
    if bps < 1024 {
        return format!("{bps}B");
    }
    if bps < 1024 * 1024 {
        return format!("{:.0}K", bps as f64 / 1024.0);
    }
    if bps < 1024_u64.pow(3) {
        return format!("{:.1}M", bps as f64 / (1024.0 * 1024.0));
    }
    format!("{:.1}G", bps as f64 / 1024_f64.powi(3))
}

/// Two equal columns with a fixed total width so wrapped text cannot expand the card.
fn device_card_two_col_row(
    ui: &mut egui::Ui,
    inner_w: f32,
    gap_x: f32,
    left: RichText,
    right: RichText,
) {
    let col_w = ((inner_w - gap_x).max(1.0) * 0.5).max(1.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap_x;
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.set_width(col_w);
            ui.add(Label::new(left).wrap());
        });
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.set_width(col_w);
            ui.add(Label::new(right).wrap());
        });
    });
}

/// Horizontal gap between label and value in stats rows (`format!("{} {:.0}%", …)` uses one space).
fn device_card_stat_label_value_gap(ui: &egui::Ui, font_px: f32) -> f32 {
    WidgetText::from(RichText::new(" ").size(font_px))
        .into_galley(ui, None, f32::INFINITY, TextStyle::Body)
        .size()
        .x
}

/// Remark row: title + same label–value gap as memory line, then content; full-width hit rect.
fn device_mgmt_remark_row_interact(
    ui: &mut egui::Ui,
    inner_w: f32,
    label_value_gap: f32,
    title_rt: RichText,
    right_rt: RichText,
    interact_id: egui::Id,
    min_row_h: f32,
) -> egui::Response {
    // `ui.horizontal` vertically centers children; we need title and content top-aligned like stats rows.
    let inner = ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(Label::new(title_rt));
        ui.add_space(label_value_gap);
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            let rest = ui.available_width().max(1.0);
            ui.set_width(rest);
            ui.add(Label::new(right_rt).wrap());
        });
    });
    let mut r = inner.response.rect;
    r.max.x = r.min.x + inner_w;
    let h = r.height().max(min_row_h);
    r.max.y = r.min.y + h;
    ui.interact(r, interact_id, Sense::click())
}

/// When offline, always zeros (ignore any stale map). When online without stats yet, zeros.
fn device_card_resource_values(
    online: bool,
    stats: Option<&titan_common::HostResourceStats>,
) -> (f32, f64, String, String, String, String) {
    if !online {
        return (
            0.0,
            0.0,
            format_rate_bps_short(0),
            format_rate_bps_short(0),
            format_rate_bps_short(0),
            format_rate_bps_short(0),
        );
    }
    let Some(st) = stats else {
        return (
            0.0,
            0.0,
            format_rate_bps_short(0),
            format_rate_bps_short(0),
            format_rate_bps_short(0),
            format_rate_bps_short(0),
        );
    };
    let cpu_pct = (st.cpu_permille.min(1000) as f32) / 10.0;
    let mem_pct = if st.mem_total_bytes > 0 {
        ((st.mem_used_bytes as f64 * 100.0) / st.mem_total_bytes as f64).min(100.0)
    } else {
        0.0
    };
    (
        cpu_pct,
        mem_pct,
        format_rate_bps_short(st.net_down_bps),
        format_rate_bps_short(st.net_up_bps),
        format_rate_bps_short(st.disk_read_bps),
        format_rate_bps_short(st.disk_write_bps),
    )
}

fn card_outline(ui: &egui::Ui) -> Stroke {
    Stroke::new(
        1.0,
        ui.visuals()
            .widgets
            .noninteractive
            .bg_stroke
            .color
            .linear_multiply(0.55),
    )
}

/// Columns in \[1, 6\]; card width clamped to \[DEVICE_CARD_MIN_WIDTH, DEVICE_CARD_MAX_WIDTH\] when cols > 1.
fn device_mgmt_cols_and_card_width(available: f32) -> (usize, f32) {
    let gap = DEVICE_CARD_GAP;
    let min_w = DEVICE_CARD_MIN_WIDTH;
    let max_w = DEVICE_CARD_MAX_WIDTH;

    if available <= 0.0 {
        return (1, 100.0);
    }

    let max_cols_fit = ((available + gap) / (min_w + gap)).floor() as usize;
    let mut cols = max_cols_fit.clamp(1, 6);

    let raw_card_w = |c: usize| -> f32 {
        if c <= 1 {
            available
        } else {
            (available - gap * (c - 1) as f32) / c as f32
        }
    };

    let mut card_w = raw_card_w(cols);

    while card_w > max_w && cols < 6 {
        cols += 1;
        card_w = raw_card_w(cols);
    }
    while card_w < min_w && cols > 1 {
        cols -= 1;
        card_w = raw_card_w(cols);
    }

    if cols <= 1 {
        card_w = available.min(max_w).max(80.0);
    } else {
        card_w = card_w.clamp(min_w, max_w);
    }
    card_w = card_w.min(max_w);

    (cols, card_w)
}

/// Used to seed masonry column heights before the first paint for each device row.
fn device_mgmt_card_height_hint(card_w: f32) -> f32 {
    let preview_h = (card_w * 9.0 / 16.0).clamp(100.0, 200.0);
    preview_h + 200.0
}

impl CenterApp {
    /// Device management: cards sit directly in the page scroll (no inner list container).
    /// Each card is **vertical**: full-bleed desktop preview on top (no inner frame), padded text block below (title → status/stats → address).
    pub(super) fn panel_device_management(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        ui.spacing_mut().item_spacing.y = 10.0;

        ui.horizontal(|ui| {
            if subtle_button(ui, t(lang, Msg::BtnAddHost), true).clicked() {
                self.add_host_dialog_ip = discovery::default_manual_host_ipv4_string();
                self.add_host_dialog_port = "7788".into();
                self.add_host_dialog_err.clear();
                self.add_host_dialog_open = true;
            }
            if subtle_button(
                ui,
                t(lang, Msg::BtnRemoveSelected),
                !self.endpoints.is_empty(),
            )
            .clicked()
            {
                let idx = self
                    .selected_host
                    .min(self.endpoints.len().saturating_sub(1));
                if !self.endpoints.is_empty() {
                    self.device_remark_edit_index = None;
                    self.device_remark_edit_focus_next = false;
                    self.endpoints.remove(idx);
                    self.selected_host = self.selected_host.saturating_sub(1);
                }
            }
            if subtle_button(
                ui,
                t(lang, Msg::BtnHostHello),
                !self.fleet_busy && !self.endpoints.is_empty(),
            )
            .clicked()
            {
                self.spawn_fleet_hello_selected();
            }
            if subtle_button(
                ui,
                t(lang, Msg::BtnHostTelemetry),
                !self.endpoints.is_empty(),
            )
            .clicked()
            {
                self.spawn_fleet_telemetry_selected();
            }
        });
        ui.add_space(12.0);

        if self.endpoints.is_empty() {
            let w = ui.available_width();
            let h = ui.available_height().max(180.0);
            ui.allocate_ui_with_layout(egui::vec2(w, h), Layout::top_down(Align::Min), |ui| {
                let rect = ui.max_rect();
                let text_width = (w * 0.92).clamp(1.0, 520.0);
                let main_color = ui.visuals().widgets.inactive.text_color();
                let hint_color = ui.visuals().weak_text_color();
                let main_galley = WidgetText::from(
                    RichText::new(t(lang, Msg::DeviceMgmtNoRegistered))
                        .size(15.0)
                        .color(main_color),
                )
                .into_galley(
                    ui,
                    Some(TextWrapMode::Wrap),
                    text_width,
                    TextStyle::Body,
                );
                let hint_galley = WidgetText::from(
                    RichText::new(t(lang, Msg::DeviceMgmtEmptyHint))
                        .small()
                        .line_height(Some(20.0))
                        .color(hint_color),
                )
                .into_galley(
                    ui,
                    Some(TextWrapMode::Wrap),
                    text_width,
                    TextStyle::Small,
                );
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
            });
        } else {
            let inner = ui.available_width();
            let (cols, card_w) = device_mgmt_cols_and_card_width(inner);
            let gap = DEVICE_CARD_GAP;
            let row_w = cols as f32 * card_w + (cols.saturating_sub(1) as f32) * gap;
            let lead = ((inner - row_w).max(0.0)) * 0.5;

            let n = self.endpoints.len();
            const CARD_STACK_GAP: f32 = 14.0;
            self.device_masonry_heights.retain(|k, _| {
                self.endpoints
                    .iter()
                    .any(|e| Self::endpoint_addr_key(&e.addr) == *k)
            });
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
                col_load[c] += est + CARD_STACK_GAP;
            }

            let grid_tl = ui.cursor().min;
            let start_x = grid_tl.x + lead;
            let y0 = grid_tl.y;
            let mut col_y = vec![y0; cols];
            let col_x: Vec<f32> = (0..cols)
                .map(|c| start_x + c as f32 * (card_w + gap))
                .collect();

            for c in 0..cols {
                for &i in &columns[c] {
                    let addr_key = Self::endpoint_addr_key(&self.endpoints[i].addr);
                    let mut slot_h = self
                        .device_masonry_heights
                        .get(&addr_key)
                        .copied()
                        .unwrap_or(0.0);
                    if slot_h < 8.0 {
                        slot_h = device_mgmt_card_height_hint(card_w);
                    }
                    let rect = Rect::from_min_size(
                        pos2(col_x[c], col_y[c]),
                        Vec2::new(card_w, slot_h),
                    );
                    let slot = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                        let is_sel = self.selected_host == i;
                        let (label_s, addr_s, win_n, online) = {
                            let ep = &self.endpoints[i];
                            (
                                ep.label.clone(),
                                ep.addr.clone(),
                                ep.last_vm_count,
                                ep.last_known_online,
                            )
                        };

                        let stroke = card_outline(ui);

                        Frame::NONE
                            .fill(CARD_SURFACE)
                            .stroke(stroke)
                            .corner_radius(CARD_CORNER_RADIUS)
                            .shadow(card_shadow())
                            .inner_margin(Margin::ZERO)
                            .show(ui, |ui| {
                                ui.set_width(card_w);
                                ui.set_min_width(card_w);
                                ui.set_max_width(card_w);
                                let card_tl = ui.cursor().min;
                                let mut select_split_y = card_tl.y;
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let preview_key = Self::endpoint_addr_key(&addr_s);
                                    let preview_h = (card_w * 9.0 / 16.0).clamp(100.0, 200.0);
                                    let (preview_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(card_w, preview_h),
                                        Sense::empty(),
                                    );
                                    let preview_corners = CornerRadius {
                                        nw: CARD_CORNER_RADIUS.nw,
                                        ne: CARD_CORNER_RADIUS.ne,
                                        sw: 0,
                                        se: 0,
                                    };
                                    if let Some(tex) = self.host_desktop_textures.get(&preview_key)
                                    {
                                        // Host JPEG keeps display aspect (e.g. 16:10 → 576×360); card slot is 16:9.
                                        // egui defaults to contain-fit inside `Exact`, which pillarboxes — turn off to fill.
                                        ui.put(
                                            preview_rect,
                                            egui::Image::new(tex)
                                                .corner_radius(preview_corners)
                                                .fit_to_exact_size(preview_rect.size())
                                                .maintain_aspect_ratio(false),
                                        );
                                    } else {
                                        ui.painter().rect_filled(
                                            preview_rect,
                                            preview_corners,
                                            DEVICE_PREVIEW_PLACEHOLDER_BG,
                                        );
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
                                        ui.painter().galley(
                                            pos,
                                            galley,
                                            DEVICE_PREVIEW_PLACEHOLDER_TEXT,
                                        );
                                    }

                                    Frame::NONE.inner_margin(Margin::symmetric(12, 10)).show(
                                        ui,
                                        |ui| {
                                            let inner_w = (card_w - 24.0).max(1.0);
                                            ui.set_width(inner_w);
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing.y = 6.0;
                                                let st_ref = online
                                                    .then(|| {
                                                        self.host_resource_stats.get(&preview_key)
                                                    })
                                                    .flatten();
                                                let (
                                                    cpu_pct,
                                                    mem_pct,
                                                    net_down,
                                                    net_up,
                                                    disk_r,
                                                    disk_w,
                                                ) = device_card_resource_values(online, st_ref);
                                                let weak =
                                                    ui.visuals().widgets.inactive.text_color();
                                                let title_color = if is_sel {
                                                    ACCENT
                                                } else {
                                                    ui.visuals().strong_text_color()
                                                };
                                                const CARD_STATUS_TITLE_PX: f32 = 16.0;
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 6.0;
                                                    if online {
                                                        ui.label(
                                                            RichText::new("●")
                                                                .size(CARD_STATUS_TITLE_PX)
                                                                .color(OK_GREEN),
                                                        );
                                                        ui.label(
                                                            RichText::new(t(
                                                                lang,
                                                                Msg::MonitorStatOnline,
                                                            ))
                                                            .size(CARD_STATUS_TITLE_PX)
                                                            .color(weak),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            RichText::new("○")
                                                                .size(CARD_STATUS_TITLE_PX)
                                                                .weak(),
                                                        );
                                                        ui.label(
                                                            RichText::new(t(
                                                                lang,
                                                                Msg::MonitorStatOffline,
                                                            ))
                                                            .size(CARD_STATUS_TITLE_PX)
                                                            .weak(),
                                                        );
                                                    }
                                                    ui.add(
                                                        Label::new(
                                                            RichText::new(&label_s)
                                                                .strong()
                                                                .size(CARD_STATUS_TITLE_PX)
                                                                .color(title_color),
                                                        )
                                                        .truncate(),
                                                    );
                                                });
                                                const CARD_BODY_GRID_PX: f32 = 13.0;
                                                let addr_win_color =
                                                    ui.visuals().widgets.inactive.text_color();
                                                const BODY_ROW_GAP: f32 = 5.0;
                                                device_card_two_col_row(
                                                    ui,
                                                    inner_w,
                                                    DEVICE_CARD_BODY_COL_GAP,
                                                    RichText::new(format!(
                                                        "{} {:.1}%",
                                                        t(lang, Msg::DeviceMgmtResCpu),
                                                        cpu_pct
                                                    ))
                                                    .size(CARD_BODY_GRID_PX)
                                                    .color(weak),
                                                    RichText::new(format!(
                                                        "{} {} / {}",
                                                        t(lang, Msg::DeviceMgmtResNet),
                                                        net_down,
                                                        net_up
                                                    ))
                                                    .size(CARD_BODY_GRID_PX)
                                                    .color(weak),
                                                );
                                                ui.add_space(BODY_ROW_GAP);
                                                device_card_two_col_row(
                                                    ui,
                                                    inner_w,
                                                    DEVICE_CARD_BODY_COL_GAP,
                                                    RichText::new(format!(
                                                        "{} {:.0}%",
                                                        t(lang, Msg::DeviceMgmtResMem),
                                                        mem_pct
                                                    ))
                                                    .size(CARD_BODY_GRID_PX)
                                                    .color(weak),
                                                    RichText::new(format!(
                                                        "{} {} / {}",
                                                        t(lang, Msg::DeviceMgmtResDiskIo),
                                                        disk_r,
                                                        disk_w
                                                    ))
                                                    .size(CARD_BODY_GRID_PX)
                                                    .color(weak),
                                                );
                                                ui.add_space(BODY_ROW_GAP);
                                                device_card_two_col_row(
                                                    ui,
                                                    inner_w,
                                                    DEVICE_CARD_BODY_COL_GAP,
                                                    RichText::new(&addr_s)
                                                        .monospace()
                                                        .size(CARD_BODY_GRID_PX)
                                                        .color(addr_win_color),
                                                    RichText::new(host_running_windows_line(
                                                        lang, win_n,
                                                    ))
                                                    .size(CARD_BODY_GRID_PX)
                                                    .color(addr_win_color),
                                                );
                                                ui.add_space(BODY_ROW_GAP);
                                                select_split_y = ui.cursor().min.y;

                                                /// Fixed row height: avoids card growing when the underline field gains focus.
                                                const REMARK_ROW_H: f32 = 32.0;
                                                let weak = ui.visuals().widgets.inactive.text_color();
                                                let remark_font =
                                                    FontId::proportional(CARD_BODY_GRID_PX);
                                                let title_rt =
                                                    RichText::new(t(lang, Msg::DeviceMgmtRemarkTitle))
                                                        .font(remark_font.clone())
                                                        .color(weak);
                                                let stat_lbl_gap =
                                                    device_card_stat_label_value_gap(
                                                        ui,
                                                        CARD_BODY_GRID_PX,
                                                    );
                                                let editing = self.device_remark_edit_index == Some(i);
                                                if editing {
                                                    let edit_id = ui.make_persistent_id((
                                                        "device_mgmt_remark_edit",
                                                        i,
                                                    ));
                                                    let request_focus = std::mem::take(
                                                        &mut self.device_remark_edit_focus_next,
                                                    );
                                                    let end_edit = Cell::new(false);
                                                    ui.allocate_ui_with_layout(
                                                        egui::vec2(inner_w, REMARK_ROW_H),
                                                        Layout::left_to_right(Align::Min),
                                                        |ui| {
                                                            ui.spacing_mut().item_spacing.x = 0.0;
                                                            ui.add(Label::new(title_rt.clone()));
                                                            ui.add_space(stat_lbl_gap);
                                                            ui.with_layout(
                                                                Layout::top_down(Align::Min),
                                                                |ui| {
                                                                    ui.set_width(ui.available_width());
                                                                    let te_resp =
                                                                        dialog_underline_text_row_gap(
                                                                            ui,
                                                                            |ui| {
                                                                                egui::TextEdit::singleline(
                                                                                    &mut self.endpoints[i].remark,
                                                                                )
                                                                                .id(edit_id)
                                                                                .frame(false)
                                                                                .background_color(
                                                                                    Color32::TRANSPARENT,
                                                                                )
                                                                                .margin(Margin::symmetric(0, 4))
                                                                                .font(remark_font.clone())
                                                                                .desired_width(ui.available_width())
                                                                                .hint_text(
                                                                                    RichText::new(t(
                                                                                        lang,
                                                                                        Msg::DeviceMgmtRemarkDblclkHint,
                                                                                    ))
                                                                                    .font(remark_font.clone())
                                                                                    .color(ADD_HOST_DLG_MUTED),
                                                                                )
                                                                                .text_color(ADD_HOST_DLG_BODY)
                                                                                .show(ui)
                                                                            },
                                                                            0.0,
                                                                        );
                                                                    if request_focus {
                                                                        te_resp.request_focus();
                                                                    }
                                                                    if te_resp.lost_focus() {
                                                                        end_edit.set(true);
                                                                    }
                                                                },
                                                            );
                                                        },
                                                    );
                                                    if end_edit.get() {
                                                        self.device_remark_edit_index = None;
                                                        self.persist_registered_devices();
                                                    }
                                                } else {
                                                    let hint = t(lang, Msg::DeviceMgmtRemarkDblclkHint);
                                                    let rem = self.endpoints[i].remark.as_str();
                                                    let right_rt = if rem.is_empty() {
                                                        RichText::new(hint)
                                                            .font(remark_font.clone())
                                                            .color(weak)
                                                    } else {
                                                        RichText::new(rem)
                                                            .font(remark_font.clone())
                                                            .color(weak)
                                                    };
                                                    let touch_id = ui.make_persistent_id((
                                                        "device_mgmt_remark_touch",
                                                        i,
                                                    ));
                                                    let row_resp = device_mgmt_remark_row_interact(
                                                        ui,
                                                        inner_w,
                                                        stat_lbl_gap,
                                                        title_rt,
                                                        right_rt,
                                                        touch_id,
                                                        REMARK_ROW_H,
                                                    );
                                                    if row_resp.double_clicked() {
                                                        self.device_remark_edit_index = Some(i);
                                                        self.device_remark_edit_focus_next = true;
                                                    }
                                                }
                                            });
                                        },
                                    );
                                });
                                let card_br = ui.min_rect().max;
                                let select_rect =
                                    Rect::from_min_max(card_tl, pos2(card_br.x, select_split_y));
                                ui.interact(
                                    select_rect,
                                    ui.make_persistent_id(("device_mgmt_card", i)),
                                    Sense::click(),
                                )
                            })
                    });

                    if slot.inner.inner.clicked() {
                        self.select_endpoint_host(i);
                    }
                    let used = slot.inner.response.rect.height().max(32.0);
                    self.device_masonry_heights
                        .insert(addr_key.clone(), used);
                    col_y[c] += used + CARD_STACK_GAP;
                }
            }

            let max_bottom = (0..cols)
                .map(|c| {
                    if columns[c].is_empty() {
                        y0
                    } else {
                        col_y[c] - CARD_STACK_GAP
                    }
                })
                .fold(y0, f32::max);
            let total_h = (max_bottom - y0).max(0.0);
            ui.allocate_space(Vec2::new(lead + row_w, total_h));
        }

        self.show_add_host_dialog(ui, lang);
    }

    fn show_add_host_dialog(&mut self, ui: &mut egui::Ui, lang: super::i18n::UiLang) {
        if !self.add_host_dialog_open {
            return;
        }
        let mut win_open = self.add_host_dialog_open;
        let mut force_close = false;
        let ctx = ui.ctx().clone();
        let title = t(lang, Msg::AddHostDialogTitle);
        const DIALOG_INNER: Vec2 = Vec2::new(440.0, 312.0);
        egui::Window::new(title)
            .id(egui::Id::new("titan_center_add_host_dialog"))
            .frame(opaque_dialog_frame(ui))
            .open(&mut win_open)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .fade_in(false)
            .fade_out(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(DIALOG_INNER)
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                let full_w = ui.available_width();
                ui.set_width(full_w);
                ui.spacing_mut().item_spacing.y = 0.0;

                ui.add(
                    egui::Label::new(
                        RichText::new(t(lang, Msg::AddHostDialogSubtitle))
                            .size(12.5)
                            .line_height(Some(18.0))
                            .color(ADD_HOST_DLG_MUTED),
                    )
                    .wrap(),
                );
                ui.add_space(16.0);

                ui.label(
                    RichText::new(t(lang, Msg::AddHostIpLabel))
                        .size(13.0)
                        .strong()
                        .color(ADD_HOST_DLG_LABEL),
                );
                ui.add_space(6.0);
                dialog_underline_text_row(ui, |ui| {
                    egui::TextEdit::singleline(&mut self.add_host_dialog_ip)
                        .frame(false)
                        .background_color(Color32::TRANSPARENT)
                        .margin(Margin::symmetric(0, 8))
                        .desired_width(ui.available_width())
                        .font(TextStyle::Monospace)
                        .hint_text(RichText::new("192.168.1.1").color(ADD_HOST_DLG_MUTED))
                        .text_color(ADD_HOST_DLG_BODY)
                        .show(ui)
                });
                ui.add_space(14.0);

                ui.label(
                    RichText::new(t(lang, Msg::AddHostPortLabel))
                        .size(13.0)
                        .strong()
                        .color(ADD_HOST_DLG_LABEL),
                );
                ui.add_space(6.0);
                dialog_underline_text_row(ui, |ui| {
                    egui::TextEdit::singleline(&mut self.add_host_dialog_port)
                        .frame(false)
                        .background_color(Color32::TRANSPARENT)
                        .margin(Margin::symmetric(0, 8))
                        .desired_width(ui.available_width())
                        .font(TextStyle::Monospace)
                        .hint_text(RichText::new("7788").color(ADD_HOST_DLG_MUTED))
                        .text_color(ADD_HOST_DLG_BODY)
                        .show(ui)
                });

                ui.add_space(12.0);
                if !self.add_host_dialog_err.is_empty() {
                    Frame::NONE
                        .fill(ADD_HOST_ERR_BG)
                        .stroke(Stroke::new(1.0, ADD_HOST_ERR_BORDER))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&self.add_host_dialog_err)
                                        .size(12.5)
                                        .line_height(Some(18.0))
                                        .color(ADD_HOST_ERR_TEXT),
                                )
                                .wrap(),
                            );
                        });
                }

                if self.add_host_verify_busy {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(t(lang, Msg::AddHostVerifying))
                            .size(13.0)
                            .color(ADD_HOST_DLG_MUTED),
                    );
                }

                ui.add_space(20.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(full_w, 48.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        if primary_button_large(
                            ui,
                            t(lang, Msg::AddHostConfirm),
                            !self.add_host_verify_busy,
                        )
                        .clicked()
                        {
                            let ip_s = self.add_host_dialog_ip.trim();
                            let port_s = self.add_host_dialog_port.trim();
                            let ip_ok = Ipv4Addr::from_str(ip_s).ok();
                            let port_ok: Option<u16> = port_s.parse().ok().filter(|&p| p > 0);
                            if let (Some(ip), Some(port)) = (ip_ok, port_ok) {
                                let addr = format!("{ip}:{port}");
                                self.spawn_add_host_verify(addr);
                                self.add_host_dialog_err.clear();
                            } else {
                                self.add_host_dialog_err =
                                    t(lang, Msg::AddHostInvalidHint).to_string();
                            }
                        }
                        if subtle_button_large(ui, t(lang, Msg::BtnCancel), true).clicked() {
                            force_close = true;
                            self.add_host_dialog_err.clear();
                        }
                    },
                );
            });
        if force_close {
            win_open = false;
        }
        self.add_host_dialog_open = win_open;
        if !self.add_host_dialog_open {
            self.add_host_dialog_err.clear();
            self.invalidate_add_host_probe();
        }
    }
}
