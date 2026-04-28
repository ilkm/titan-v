//! Inset-styled single-select dropdown; trigger matches inset multiline / text field shells.

use egui::scroll_area::ScrollArea;
use egui::{
    AboveOrBelow, Align2, CornerRadius, Frame, Id, InnerResponse, Margin, PopupCloseBehavior,
    Response, Sense, Shape, Stroke, TextStyle, TextWrapMode, Ui, Vec2, WidgetText,
};

use crate::theme::FORM_VALUE_TEXT;

/// Trigger sizing for [`inset_single_select_dropdown`].
#[derive(Clone, Copy)]
pub struct InsetDropdownLayout {
    pub row_height: f32,
    pub min_trigger_width: f32,
    pub below_trigger_spacing: f32,
    pub trigger_margin: Margin,
    pub chevron_side: f32,
    pub text_right_reserve: f32,
}

impl Default for InsetDropdownLayout {
    fn default() -> Self {
        Self {
            row_height: 30.0,
            min_trigger_width: 120.0,
            below_trigger_spacing: 6.0,
            trigger_margin: Margin::symmetric(10, 8),
            chevron_side: 18.0,
            text_right_reserve: 22.0,
        }
    }
}

impl InsetDropdownLayout {
    /// Tighter trigger row and margins (e.g. language popup in settings).
    pub fn compact() -> Self {
        Self {
            row_height: 24.0,
            min_trigger_width: 88.0,
            below_trigger_spacing: 4.0,
            trigger_margin: Margin::symmetric(8, 6),
            chevron_side: 15.0,
            text_right_reserve: 19.0,
        }
    }
}

fn widget_to_popup_id(button_id: Id) -> Id {
    button_id.with("titan_inset_dropdown_popup")
}

/// Prefer opening below; flip above when the menu would clip off the bottom of the screen.
fn above_or_below_for_dropdown(ui: &Ui, menu_max_height: f32) -> AboveOrBelow {
    let popup_h = menu_max_height.clamp(72.0, 280.0);
    if ui.next_widget_position().y + ui.spacing().interact_size.y + popup_h
        < ui.ctx().screen_rect().bottom()
    {
        AboveOrBelow::Below
    } else {
        AboveOrBelow::Above
    }
}

fn trigger_inset_frame(ui: &Ui, trigger_margin: Margin) -> Frame {
    let fill = ui
        .visuals()
        .widgets
        .noninteractive
        .bg_fill
        .linear_multiply(1.08);
    let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    Frame::NONE
        .fill(fill)
        .corner_radius(CornerRadius::same(8))
        .stroke(stroke)
        .inner_margin(trigger_margin)
}

fn paint_chevron(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let s = rect.height().min(rect.width()) * 0.38;
    let c = rect.center() + Vec2::new(0.0, 1.0);
    let pts = vec![
        c + Vec2::new(-s, -s * 0.35),
        c + Vec2::new(s, -s * 0.35),
        c + Vec2::new(0.0, s * 0.65),
    ];
    painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

fn paint_trigger_galley(ui: &Ui, rect: egui::Rect, selected_text: WidgetText, text_w: f32) {
    let galley =
        selected_text.into_galley(ui, Some(TextWrapMode::Truncate), text_w, TextStyle::Body);
    let pos = Align2::LEFT_CENTER
        .align_size_within_rect(galley.size(), rect.shrink2(Vec2::new(0.0, 2.0)));
    ui.painter().galley(pos.min, galley, FORM_VALUE_TEXT);
}

fn trigger_row_response(
    ui: &mut Ui,
    popup_id: Id,
    w: f32,
    selected_text: WidgetText,
    layout: &InsetDropdownLayout,
) -> Response {
    ui.set_width(w);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, layout.row_height), Sense::click());
    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    if ui.is_rect_visible(rect) {
        let text_w = (rect.width() - layout.text_right_reserve).max(8.0);
        paint_trigger_galley(ui, rect, selected_text, text_w);
        let chevron_rect =
            Align2::RIGHT_CENTER.align_size_within_rect(Vec2::splat(layout.chevron_side), rect);
        paint_chevron(
            ui.painter(),
            chevron_rect,
            FORM_VALUE_TEXT.linear_multiply(0.55),
        );
    }
    response
}

fn show_inset_dropdown_trigger(
    ui: &mut Ui,
    popup_id: Id,
    w: f32,
    selected_text: WidgetText,
    layout: &InsetDropdownLayout,
) -> Response {
    trigger_inset_frame(ui, layout.trigger_margin)
        .show(ui, |ui| {
            trigger_row_response(ui, popup_id, w, selected_text, layout)
        })
        .inner
}

fn dropdown_popup<R>(
    ui: &Ui,
    popup_id: Id,
    trigger_response: &Response,
    above_or_below: AboveOrBelow,
    menu_max_height: f32,
    menu_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    egui::popup::popup_above_or_below_widget(
        ui,
        popup_id,
        trigger_response,
        above_or_below,
        PopupCloseBehavior::CloseOnClick,
        |ui| {
            ScrollArea::vertical()
                .max_height(menu_max_height)
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    menu_contents(ui)
                })
                .inner
        },
    )
}

/// Inset field–style trigger + scrollable popup; same interaction model as [`egui::ComboBox`].
pub fn inset_single_select_dropdown<R>(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    trigger_width: f32,
    selected_text: impl Into<WidgetText>,
    menu_max_height: f32,
    layout: InsetDropdownLayout,
    menu_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<Option<R>> {
    let button_id = ui.make_persistent_id(id_salt);
    let popup_id = widget_to_popup_id(button_id);
    let above_or_below = above_or_below_for_dropdown(ui, menu_max_height);
    let w = trigger_width.max(layout.min_trigger_width);
    let selected_text: WidgetText = selected_text.into();
    let trigger_response = show_inset_dropdown_trigger(ui, popup_id, w, selected_text, &layout);
    ui.add_space(layout.below_trigger_spacing);
    let inner = dropdown_popup(
        ui,
        popup_id,
        &trigger_response,
        above_or_below,
        menu_max_height,
        menu_contents,
    );
    InnerResponse {
        inner,
        response: trigger_response,
    }
}
