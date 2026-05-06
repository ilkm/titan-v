use egui::{Color32, Label, RichText};

use super::super::helpers_dup::{
    DEVICE_CARD_BODY_COL_GAP, device_card_resource_values, device_card_two_col_row,
};
use super::{CARD_BODY_GRID_PX, METRIC_BODY_ROW_GAP};
use crate::host_app::constants::ACCENT;
use crate::host_app::model::HostApp;
use crate::titan_i18n::{Msg, UiLang, host_running_windows_line, t};

const WARN_YELLOW: Color32 = Color32::from_rgb(245, 158, 11);

pub(super) fn paint_device_status_and_metrics(
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
    let title_color = if is_sel {
        ACCENT
    } else {
        ui.visuals().strong_text_color()
    };
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
            ui.label(RichText::new("●").size(px).color(WARN_YELLOW));
            ui.label(
                RichText::new(t(lang, Msg::MonitorStatNotBooted))
                    .size(px)
                    .color(weak),
            );
        } else {
            ui.label(RichText::new("○").size(px).weak());
            ui.label(
                RichText::new(t(lang, Msg::MonitorStatOffline))
                    .size(px)
                    .weak(),
            );
        }
        ui.add(Label::new(RichText::new(label_s).strong().size(px).color(title_color)).truncate());
    });
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
    paint_metric_row_cpu_net(ui, lang, inner_w, weak, cpu_pct, net_down, net_up);
    ui.add_space(METRIC_BODY_ROW_GAP);
    paint_metric_row_mem_disk(ui, lang, inner_w, weak, mem_pct, disk_r, disk_w);
    ui.add_space(METRIC_BODY_ROW_GAP);
    paint_metric_row_addr_win(ui, lang, inner_w, addr_s, win_n);
    ui.add_space(METRIC_BODY_ROW_GAP);
}

fn paint_metric_row_cpu_net(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    cpu_pct: f32,
    net_down: String,
    net_up: String,
) {
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
}

fn paint_metric_row_mem_disk(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    weak: Color32,
    mem_pct: f64,
    disk_r: String,
    disk_w: String,
) {
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
}

fn paint_metric_row_addr_win(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    addr_s: &str,
    win_n: u32,
) {
    let color = ui.visuals().widgets.inactive.text_color();
    device_card_two_col_row(
        ui,
        inner_w,
        DEVICE_CARD_BODY_COL_GAP,
        RichText::new(addr_s)
            .monospace()
            .size(CARD_BODY_GRID_PX)
            .color(color),
        RichText::new(host_running_windows_line(lang, win_n))
            .size(CARD_BODY_GRID_PX)
            .color(color),
    );
}
