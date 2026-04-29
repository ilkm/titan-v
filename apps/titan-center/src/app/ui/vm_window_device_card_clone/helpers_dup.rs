//! **Fork** of `devices/helpers.rs` (subset used by window VM cards). Edit independently of Connect tab.

use egui::{Align, Color32, Label, Layout, RichText, Sense, Stroke, TextStyle, WidgetText};

/// Horizontal gap between the two text columns on device cards (CPU/mem, disk, remark…).
pub(super) const DEVICE_CARD_BODY_COL_GAP: f32 = 10.0;

pub(super) const DEVICE_PREVIEW_PLACEHOLDER_BG: Color32 = Color32::from_rgb(236, 238, 242);
pub(super) const DEVICE_PREVIEW_PLACEHOLDER_TEXT: Color32 = Color32::BLACK;

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

pub(super) fn device_card_stat_label_value_gap(ui: &egui::Ui, font_px: f32) -> f32 {
    WidgetText::from(RichText::new(" ").size(font_px))
        .into_galley(ui, None, f32::INFINITY, TextStyle::Body)
        .size()
        .x
}

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
