//! Layout helpers for device management cards (Connect tab).

use egui::{Align, Color32, Label, Layout, RichText, Sense, Stroke, TextStyle, WidgetText};

use crate::app::constants::{DEVICE_CARD_GAP, DEVICE_CARD_MAX_WIDTH, DEVICE_CARD_MIN_WIDTH};

/// Horizontal gap between the two text columns on device cards (CPU/mem, disk, remark…).
pub(super) const DEVICE_CARD_BODY_COL_GAP: f32 = 10.0;

/// Desktop preview slot when no JPEG has been received yet (unified look across cards).
pub(super) const DEVICE_PREVIEW_PLACEHOLDER_BG: Color32 = Color32::from_rgb(236, 238, 242);
pub(super) const DEVICE_PREVIEW_PLACEHOLDER_TEXT: Color32 = Color32::BLACK;

/// Add-host modal typography on white (light theme).
pub(super) const ADD_HOST_DLG_BODY: Color32 = Color32::from_rgb(15, 23, 42);
pub(super) const ADD_HOST_DLG_MUTED: Color32 = Color32::from_rgb(71, 85, 105);
pub(super) const ADD_HOST_DLG_LABEL: Color32 = Color32::from_rgb(51, 65, 85);
pub(super) const ADD_HOST_ERR_BG: Color32 = Color32::from_rgb(255, 241, 242);
pub(super) const ADD_HOST_ERR_BORDER: Color32 = Color32::from_rgb(254, 202, 202);
pub(super) const ADD_HOST_ERR_TEXT: Color32 = Color32::from_rgb(185, 28, 28);

/// Compact rates for narrow device cards (K/M suffix, no `/s` — keeps columns narrow).
pub(super) fn format_rate_bps_short(bps: u64) -> String {
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
pub(super) fn device_card_two_col_row(
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
pub(super) fn device_card_stat_label_value_gap(ui: &egui::Ui, font_px: f32) -> f32 {
    WidgetText::from(RichText::new(" ").size(font_px))
        .into_galley(ui, None, f32::INFINITY, TextStyle::Body)
        .size()
        .x
}

/// Remark row: title + same label–value gap as memory line, then content; full-width hit rect.
pub(super) fn device_mgmt_remark_row_interact(
    ui: &mut egui::Ui,
    inner_w: f32,
    label_value_gap: f32,
    title_rt: RichText,
    right_rt: RichText,
    interact_id: egui::Id,
    min_row_h: f32,
) -> egui::Response {
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

fn device_card_resource_tuple_zeros() -> (f32, f64, String, String, String, String) {
    (
        0.0,
        0.0,
        format_rate_bps_short(0),
        format_rate_bps_short(0),
        format_rate_bps_short(0),
        format_rate_bps_short(0),
    )
}

/// When offline, always zeros (ignore any stale map). When online without stats yet, zeros.
pub(super) fn device_card_resource_values(
    online: bool,
    stats: Option<&titan_common::HostResourceStats>,
) -> (f32, f64, String, String, String, String) {
    if !online {
        return device_card_resource_tuple_zeros();
    }
    let Some(st) = stats else {
        return device_card_resource_tuple_zeros();
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

pub(super) fn card_outline(ui: &egui::Ui) -> Stroke {
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

fn device_mgmt_raw_card_w(available: f32, gap: f32, cols: usize) -> f32 {
    if cols <= 1 {
        available
    } else {
        (available - gap * (cols - 1) as f32) / cols as f32
    }
}

fn device_mgmt_refine_cols_for_width_bounds(
    cols: &mut usize,
    card_w: &mut f32,
    available: f32,
    gap: f32,
    min_w: f32,
    max_w: f32,
) {
    while *card_w > max_w && *cols < 6 {
        *cols += 1;
        *card_w = device_mgmt_raw_card_w(available, gap, *cols);
    }
    while *card_w < min_w && *cols > 1 {
        *cols -= 1;
        *card_w = device_mgmt_raw_card_w(available, gap, *cols);
    }
}

fn device_mgmt_clamp_final_card_w(
    cols: usize,
    card_w: f32,
    available: f32,
    min_w: f32,
    max_w: f32,
) -> f32 {
    let mut w = if cols <= 1 {
        available.min(max_w).max(80.0)
    } else {
        card_w.clamp(min_w, max_w)
    };
    w = w.min(max_w);
    w
}

/// Columns in \[1, 6\]; card width clamped to \[DEVICE_CARD_MIN_WIDTH, DEVICE_CARD_MAX_WIDTH\] when cols > 1.
pub(super) fn device_mgmt_cols_and_card_width(available: f32) -> (usize, f32) {
    let gap = DEVICE_CARD_GAP;
    let min_w = DEVICE_CARD_MIN_WIDTH;
    let max_w = DEVICE_CARD_MAX_WIDTH;
    if available <= 0.0 {
        return (1, 100.0);
    }
    let max_cols_fit = ((available + gap) / (min_w + gap)).floor() as usize;
    let mut cols = max_cols_fit.clamp(1, 6);
    let mut card_w = device_mgmt_raw_card_w(available, gap, cols);
    device_mgmt_refine_cols_for_width_bounds(&mut cols, &mut card_w, available, gap, min_w, max_w);
    let card_w = device_mgmt_clamp_final_card_w(cols, card_w, available, min_w, max_w);
    (cols, card_w)
}

/// Used to seed masonry column heights before the first paint for each device row.
pub(super) fn device_mgmt_card_height_hint(card_w: f32) -> f32 {
    let preview_h = (card_w * 9.0 / 16.0).clamp(100.0, 200.0);
    preview_h + 200.0
}
