//! Standard text inputs (dialog underline row, inset multiline).

use egui::text::CursorRange;
use egui::widgets::text_edit::TextEditOutput;
use egui::{Color32, Response, Stroke, TextStyle, WidgetText, pos2};

use crate::frames::inset_editor_shell;
use crate::theme::{ACCENT, FORM_VALUE_TEXT};

const UNDERLINE_IDLE: Color32 = Color32::from_rgb(148, 163, 184);

/// Monospace multiline JSON / technical draft inside [`inset_editor_shell`].
pub fn multiline_inset(
    ui: &mut egui::Ui,
    min_height: f32,
    text: &mut String,
    hint: impl Into<WidgetText>,
) {
    let hint_text = hint.into();
    inset_editor_shell(ui, min_height, |ui| {
        egui::TextEdit::multiline(text)
            .frame(false)
            .font(TextStyle::Monospace)
            .text_color(FORM_VALUE_TEXT)
            .desired_width(ui.available_width())
            .hint_text(hint_text)
            .show(ui);
    });
}

fn dialog_underline_response_after_focus_caret(ui: &egui::Ui, output: TextEditOutput) -> Response {
    let r = output.response;
    if r.gained_focus() {
        let cursor_end = CursorRange::one(output.galley.end());
        let mut st = output.state;
        st.cursor.set_range(Some(cursor_end));
        st.store(ui.ctx(), r.id);
    }
    r
}

fn dialog_underline_paint_rule(ui: &mut egui::Ui, r: &Response, gap_below_underline: f32) {
    let y = r.rect.bottom() + 1.0;
    let color = if r.has_focus() {
        ACCENT
    } else {
        UNDERLINE_IDLE
    };
    let stroke_w = if gap_below_underline > 0.0 {
        if r.has_focus() { 1.5 } else { 1.0 }
    } else {
        1.0
    };
    ui.painter().line_segment(
        [pos2(r.rect.min.x, y), pos2(r.rect.max.x, y)],
        Stroke::new(stroke_w, color),
    );
    if gap_below_underline > 0.0 {
        ui.add_space(gap_below_underline);
    }
}

pub fn dialog_underline_text_row_gap(
    ui: &mut egui::Ui,
    add_textedit: impl FnOnce(&mut egui::Ui) -> TextEditOutput,
    gap_below_underline: f32,
) -> Response {
    let output = add_textedit(ui);
    let r = dialog_underline_response_after_focus_caret(ui, output);
    dialog_underline_paint_rule(ui, &r, gap_below_underline);
    r
}

pub fn dialog_underline_text_row(
    ui: &mut egui::Ui,
    add_textedit: impl FnOnce(&mut egui::Ui) -> TextEditOutput,
) -> Response {
    dialog_underline_text_row_gap(ui, add_textedit, 8.0)
}
