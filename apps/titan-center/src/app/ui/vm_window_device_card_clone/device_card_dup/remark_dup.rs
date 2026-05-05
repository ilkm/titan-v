use std::cell::Cell;

use egui::{Align, Color32, FontId, Label, Layout, Margin, RichText};

use super::super::helpers_dup::{
    VM_WINDOW_REMARK_HINT_COLOR, device_card_stat_label_value_gap, device_mgmt_remark_row_interact,
};
use super::{CARD_BODY_GRID_PX, REMARK_ROW_H};
use crate::app::CenterApp;
use crate::app::constants::FORM_VALUE_TEXT;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::dialog_underline_text_row_gap;

#[rustfmt::skip]
pub(super) fn paint_vm_dup_remark_block(app: &mut CenterApp, ui: &mut egui::Ui, lang: UiLang, inner_w: f32, row_ix: usize, record_id: &str) {
    let (weak, remark_font, title_rt, stat_lbl_gap) = remark_block_style(ui, lang);
    let editing = app.vm_window_remark_edit_record_id.as_deref() == Some(record_id);
    if editing {
        paint_vm_dup_remark_edit_row(app, ui, lang, inner_w, row_ix, record_id, &title_rt, &remark_font, stat_lbl_gap);
    } else {
        paint_vm_dup_remark_display_row(app, ui, lang, inner_w, row_ix, record_id, title_rt, remark_font, weak, stat_lbl_gap);
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
fn paint_vm_dup_remark_display_row(app: &mut CenterApp, ui: &mut egui::Ui, lang: UiLang, inner_w: f32, row_ix: usize, record_id: &str, title_rt: RichText, remark_font: FontId, weak: Color32, stat_lbl_gap: f32) {
    let hint = t(lang, Msg::DeviceMgmtRemarkDblclkHint);
    let rem = app
        .vm_window_records
        .get(row_ix)
        .filter(|r| r.record_id == record_id)
        .map(|r| r.remark.clone())
        .unwrap_or_default();
    let right_rt = if rem.is_empty() {
        RichText::new(hint).font(remark_font.clone()).color(weak)
    } else {
        RichText::new(rem).font(remark_font.clone()).color(weak)
    };
    let touch_id = ui.make_persistent_id(("vm_window_clone_remark_touch", record_id));
    let row_resp = device_mgmt_remark_row_interact(ui, inner_w, stat_lbl_gap, title_rt, right_rt, touch_id, REMARK_ROW_H);
    if row_resp.double_clicked() {
        app.vm_window_remark_edit_record_id = Some(record_id.to_string());
        app.vm_window_remark_edit_focus_next = true;
    }
}

#[rustfmt::skip]
fn paint_vm_dup_remark_edit_row(app: &mut CenterApp, ui: &mut egui::Ui, lang: UiLang, inner_w: f32, row_ix: usize, record_id: &str, title_rt: &RichText, remark_font: &FontId, stat_lbl_gap: f32) {
    let edit_id = ui.make_persistent_id(("vm_window_clone_remark_edit", record_id));
    let request_focus = std::mem::take(&mut app.vm_window_remark_edit_focus_next);
    let end_edit = Cell::new(false);
    ui.allocate_ui_with_layout(egui::vec2(inner_w, REMARK_ROW_H), Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(Label::new(title_rt.clone()));
        ui.add_space(stat_lbl_gap);
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.set_width(ui.available_width());
            let te_resp = vm_dup_remark_edit_field(ui, app, lang, row_ix, record_id, remark_font, edit_id);
            if request_focus {
                te_resp.request_focus();
            }
            if te_resp.lost_focus() {
                end_edit.set(true);
            }
        });
    });
    if end_edit.get() {
        app.commit_vm_window_remark_edit(record_id);
    }
}

fn vm_dup_remark_edit_field(
    ui: &mut egui::Ui,
    app: &mut CenterApp,
    lang: UiLang,
    row_ix: usize,
    record_id: &str,
    remark_font: &FontId,
    edit_id: egui::Id,
) -> egui::Response {
    let buf = vm_dup_remark_edit_buf_mut(app, row_ix, record_id);
    let hint = RichText::new(t(lang, Msg::DeviceMgmtRemarkDblclkHint))
        .font(remark_font.clone())
        .color(VM_WINDOW_REMARK_HINT_COLOR);
    dialog_underline_text_row_gap(
        ui,
        |ui| {
            egui::TextEdit::singleline(buf)
                .id(edit_id)
                .frame(false)
                .background_color(Color32::TRANSPARENT)
                .margin(Margin::symmetric(0, 4))
                .font(remark_font.clone())
                .desired_width(ui.available_width())
                .hint_text(hint)
                .text_color(FORM_VALUE_TEXT)
                .show(ui)
        },
        0.0,
    )
}

fn vm_dup_remark_edit_buf_mut<'a>(
    app: &'a mut CenterApp,
    row_ix: usize,
    record_id: &str,
) -> &'a mut String {
    let pos = app
        .vm_window_records
        .iter()
        .position(|r| r.record_id == record_id)
        .unwrap_or(row_ix.min(app.vm_window_records.len().saturating_sub(1)));
    &mut app.vm_window_records[pos].remark
}
