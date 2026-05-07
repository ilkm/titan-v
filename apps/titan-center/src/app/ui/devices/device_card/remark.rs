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

struct RemarkStyle {
    weak: Color32,
    remark_font: FontId,
    title_rt: RichText,
    stat_lbl_gap: f32,
}

#[rustfmt::skip]
pub(super) fn paint_device_remark_block(app: &mut CenterApp, ui: &mut egui::Ui, i: usize, lang: UiLang, inner_w: f32) {
    let style = remark_block_style(ui, lang);
    if app.device_remark_edit_index == Some(i) {
        paint_remark_edit_row(app, ui, i, lang, inner_w, &style);
    } else {
        paint_remark_display_row(app, ui, i, lang, inner_w, &style);
    }
}

fn remark_block_style(ui: &egui::Ui, lang: UiLang) -> RemarkStyle {
    let weak = ui.visuals().widgets.inactive.text_color();
    let remark_font = FontId::proportional(CARD_BODY_GRID_PX);
    let title_rt = RichText::new(t(lang, Msg::DeviceMgmtRemarkTitle))
        .font(remark_font.clone())
        .color(weak);
    let stat_lbl_gap = device_card_stat_label_value_gap(ui, CARD_BODY_GRID_PX);
    RemarkStyle {
        weak,
        remark_font,
        title_rt,
        stat_lbl_gap,
    }
}

fn paint_remark_edit_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    i: usize,
    lang: UiLang,
    inner_w: f32,
    style: &RemarkStyle,
) {
    let edit_id = ui.make_persistent_id(("device_mgmt_remark_edit", i));
    let request_focus = std::mem::take(&mut app.device_remark_edit_focus_next);
    if paint_remark_editor_row(app, ui, i, lang, inner_w, style, edit_id, request_focus) {
        app.device_remark_edit_index = None;
        app.persist_registered_devices();
    }
}

fn paint_remark_editor_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    i: usize,
    lang: UiLang,
    inner_w: f32,
    style: &RemarkStyle,
    edit_id: egui::Id,
    request_focus: bool,
) -> bool {
    let end_edit = Cell::new(false);
    paint_remark_editor_layout(
        app,
        ui,
        i,
        lang,
        inner_w,
        style,
        edit_id,
        request_focus,
        &end_edit,
    );
    end_edit.get()
}

fn paint_remark_editor_layout(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    i: usize,
    lang: UiLang,
    inner_w: f32,
    style: &RemarkStyle,
    edit_id: egui::Id,
    request_focus: bool,
    end_edit: &Cell<bool>,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(inner_w, REMARK_ROW_H),
        Layout::left_to_right(Align::Min),
        |ui| {
            paint_remark_editor_title(ui, style);
            paint_remark_editor_field(
                ui,
                app,
                i,
                lang,
                &style.remark_font,
                edit_id,
                request_focus,
                end_edit,
            );
        },
    );
}

fn paint_remark_editor_title(ui: &mut egui::Ui, style: &RemarkStyle) {
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.add(Label::new(style.title_rt.clone()));
    ui.add_space(style.stat_lbl_gap);
}

fn paint_remark_editor_field(
    ui: &mut egui::Ui,
    app: &mut CenterApp,
    i: usize,
    lang: UiLang,
    remark_font: &FontId,
    edit_id: egui::Id,
    request_focus: bool,
    end_edit: &Cell<bool>,
) {
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.set_width(ui.available_width());
        let te = remark_edit_underlined_field(ui, app, i, lang, remark_font, edit_id);
        if request_focus {
            te.request_focus();
        }
        if te.lost_focus() {
            end_edit.set(true);
        }
    });
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
    style: &RemarkStyle,
) {
    let right_rt = remark_display_right_rt(app, i, lang, &style.remark_font, style.weak);
    let touch_id = ui.make_persistent_id(("device_mgmt_remark_touch", i));
    let row_resp = device_mgmt_remark_row_interact(
        ui,
        inner_w,
        style.stat_lbl_gap,
        style.title_rt.clone(),
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
