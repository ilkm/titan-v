use std::cell::Cell;

use egui::{Align, Color32, FontId, Label, Layout, Margin, RichText};

use super::super::helpers::{
    ADD_HOST_DLG_MUTED, device_card_stat_label_value_gap, device_mgmt_remark_row_interact,
};
use super::{CARD_BODY_GRID_PX, REMARK_ROW_H};
use crate::app::CenterApp;
use crate::app::constants::FORM_VALUE_TEXT;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::dialog_underline_text_row_gap;

#[rustfmt::skip]
pub(super) fn paint_device_remark_block(app: &mut CenterApp, ui: &mut egui::Ui, i: usize, lang: UiLang, inner_w: f32) {
    let (weak, remark_font, title_rt, stat_lbl_gap) = remark_block_style(ui, lang);
    if app.device_remark_edit_index == Some(i) {
        paint_remark_edit_row(app, ui, i, lang, inner_w, &title_rt, &remark_font, stat_lbl_gap);
    } else {
        paint_remark_display_row(app, ui, i, lang, inner_w, title_rt, remark_font, weak, stat_lbl_gap);
    }
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

#[rustfmt::skip]
fn paint_remark_edit_row(app: &mut CenterApp, ui: &mut egui::Ui, i: usize, lang: UiLang, inner_w: f32, title_rt: &RichText, remark_font: &FontId, stat_lbl_gap: f32) {
    let edit_id = ui.make_persistent_id(("device_mgmt_remark_edit", i));
    let request_focus = std::mem::take(&mut app.device_remark_edit_focus_next);
    let end_edit = Cell::new(false);
    ui.allocate_ui_with_layout(egui::vec2(inner_w, REMARK_ROW_H), Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(Label::new(title_rt.clone()));
        ui.add_space(stat_lbl_gap);
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.set_width(ui.available_width());
            let te_resp = remark_edit_underlined_field(ui, app, i, lang, remark_font, edit_id);
            if request_focus {
                te_resp.request_focus();
            }
            if te_resp.lost_focus() {
                end_edit.set(true);
            }
        });
    });
    if end_edit.get() {
        app.device_remark_edit_index = None;
        app.persist_registered_devices();
    }
}

fn remark_edit_underlined_field(
    ui: &mut egui::Ui,
    app: &mut CenterApp,
    i: usize,
    lang: UiLang,
    remark_font: &FontId,
    edit_id: egui::Id,
) -> egui::Response {
    dialog_underline_text_row_gap(
        ui,
        |ui| {
            egui::TextEdit::singleline(&mut app.endpoints[i].remark)
                .id(edit_id)
                .frame(false)
                .background_color(Color32::TRANSPARENT)
                .margin(Margin::symmetric(0, 4))
                .font(remark_font.clone())
                .desired_width(ui.available_width())
                .hint_text(
                    RichText::new(t(lang, Msg::DeviceMgmtRemarkDblclkHint))
                        .font(remark_font.clone())
                        .color(ADD_HOST_DLG_MUTED),
                )
                .text_color(FORM_VALUE_TEXT)
                .show(ui)
        },
        0.0,
    )
}

fn paint_remark_display_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    i: usize,
    lang: UiLang,
    inner_w: f32,
    title_rt: RichText,
    remark_font: FontId,
    weak: Color32,
    stat_lbl_gap: f32,
) {
    let right_rt = remark_display_right_rt(app, i, lang, &remark_font, weak);
    let touch_id = ui.make_persistent_id(("device_mgmt_remark_touch", i));
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
        app.device_remark_edit_index = Some(i);
        app.device_remark_edit_focus_next = true;
    }
}

fn remark_display_right_rt(
    app: &CenterApp,
    i: usize,
    lang: UiLang,
    remark_font: &FontId,
    weak: Color32,
) -> RichText {
    let hint = t(lang, Msg::DeviceMgmtRemarkDblclkHint);
    let rem = app.endpoints[i].remark.as_str();
    if rem.is_empty() {
        RichText::new(hint).font(remark_font.clone()).color(weak)
    } else {
        RichText::new(rem).font(remark_font.clone()).color(weak)
    }
}
