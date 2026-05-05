use egui::{Color32, FontId, RichText};

use super::super::helpers_dup::{
    device_card_stat_label_value_gap, device_mgmt_remark_row_interact,
};
use super::{CARD_BODY_GRID_PX, REMARK_ROW_H};
use crate::titan_i18n::{Msg, UiLang, t};

pub(super) fn paint_vm_dup_remark_block(
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    record_id: &str,
    rem: &str,
) {
    if rem.trim().is_empty() {
        return;
    }
    let (weak, remark_font, title_rt, stat_lbl_gap) = remark_block_style(ui, lang);
    let right_rt = RichText::new(rem).font(remark_font).color(weak);
    let touch_id = ui.make_persistent_id(("vm_window_clone_remark_touch", record_id));
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
