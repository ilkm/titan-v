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

struct RemarkStyle {
    weak: Color32,
    remark_font: FontId,
    title_rt: RichText,
    stat_lbl_gap: f32,
}

struct VmDupEditorLayout<'a> {
    lang: UiLang,
    inner_w: f32,
    row_ix: usize,
    record_id: &'a str,
    edit_id: egui::Id,
    request_focus: bool,
    end_edit: &'a Cell<bool>,
}

pub(super) fn paint_vm_dup_remark_block(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    row_ix: usize,
    record_id: &str,
) {
    let style = remark_block_style(ui, lang);
    let editing = app.vm_window_remark_edit_record_id.as_deref() == Some(record_id);
    if editing {
        paint_vm_dup_remark_edit_row(app, ui, lang, inner_w, row_ix, record_id, &style);
    } else {
        paint_vm_dup_remark_display_row(app, ui, lang, inner_w, row_ix, record_id, &style);
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

fn paint_vm_dup_remark_display_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    row_ix: usize,
    record_id: &str,
    style: &RemarkStyle,
) {
    let hint = t(lang, Msg::DeviceMgmtRemarkDblclkHint);
    let right_rt = if let Some(rem) = vm_dup_remark_value(app, row_ix, record_id) {
        RichText::new(rem)
            .font(style.remark_font.clone())
            .color(style.weak)
    } else {
        RichText::new(hint)
            .font(style.remark_font.clone())
            .color(style.weak)
    };
    let row_resp = vm_dup_remark_row_response(ui, inner_w, record_id, style, right_rt);
    if row_resp.double_clicked() {
        app.vm_window_remark_edit_record_id = Some(record_id.to_string());
        app.vm_window_remark_edit_focus_next = true;
    }
}

fn vm_dup_remark_row_response(
    ui: &mut egui::Ui,
    inner_w: f32,
    record_id: &str,
    style: &RemarkStyle,
    right_rt: RichText,
) -> egui::Response {
    let touch_id = ui.make_persistent_id(("vm_window_clone_remark_touch", record_id));
    device_mgmt_remark_row_interact(
        ui,
        inner_w,
        style.stat_lbl_gap,
        style.title_rt.clone(),
        right_rt,
        touch_id,
        REMARK_ROW_H,
    )
}

fn paint_vm_dup_remark_edit_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    row_ix: usize,
    record_id: &str,
    style: &RemarkStyle,
) {
    let edit_id = ui.make_persistent_id(("vm_window_clone_remark_edit", record_id));
    let request_focus = std::mem::take(&mut app.vm_window_remark_edit_focus_next);
    if paint_vm_dup_remark_editor_row(
        app,
        ui,
        lang,
        inner_w,
        row_ix,
        record_id,
        style,
        edit_id,
        request_focus,
    ) {
        app.commit_vm_window_remark_edit(record_id);
    }
}

fn vm_dup_remark_value(app: &CenterApp, row_ix: usize, record_id: &str) -> Option<String> {
    app.vm_window_records
        .get(row_ix)
        .filter(|r| r.record_id == record_id)
        .map(|r| r.remark.clone())
}

fn paint_vm_dup_remark_editor_row(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    inner_w: f32,
    row_ix: usize,
    record_id: &str,
    style: &RemarkStyle,
    edit_id: egui::Id,
    request_focus: bool,
) -> bool {
    let end_edit = Cell::new(false);
    paint_vm_dup_remark_editor_layout(
        app,
        ui,
        style,
        VmDupEditorLayout {
            lang,
            inner_w,
            row_ix,
            record_id,
            edit_id,
            request_focus,
            end_edit: &end_edit,
        },
    );
    end_edit.get()
}

fn paint_vm_dup_remark_editor_layout(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    style: &RemarkStyle,
    args: VmDupEditorLayout<'_>,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(args.inner_w, REMARK_ROW_H),
        Layout::left_to_right(Align::Min),
        |ui| {
            paint_vm_dup_editor_title(ui, style);
            paint_vm_dup_editor_field(
                ui,
                app,
                args.lang,
                args.row_ix,
                args.record_id,
                &style.remark_font,
                args.edit_id,
                args.request_focus,
                args.end_edit,
            );
        },
    );
}

fn paint_vm_dup_editor_title(ui: &mut egui::Ui, style: &RemarkStyle) {
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.add(Label::new(style.title_rt.clone()));
    ui.add_space(style.stat_lbl_gap);
}

fn paint_vm_dup_editor_field(
    ui: &mut egui::Ui,
    app: &mut CenterApp,
    lang: UiLang,
    row_ix: usize,
    record_id: &str,
    remark_font: &FontId,
    edit_id: egui::Id,
    request_focus: bool,
    end_edit: &Cell<bool>,
) {
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.set_width(ui.available_width());
        let te = vm_dup_remark_edit_field(ui, app, lang, row_ix, record_id, remark_font, edit_id);
        if request_focus {
            te.request_focus();
        }
        if te.lost_focus() {
            end_edit.set(true);
        }
    });
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
