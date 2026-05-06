use egui::{
    Color32, CornerRadius, Rect, RichText, Sense, TextStyle, TextWrapMode, Vec2, WidgetText,
};

use super::super::helpers_dup::{DEVICE_PREVIEW_PLACEHOLDER_BG, DEVICE_PREVIEW_PLACEHOLDER_TEXT};
use crate::app::CenterApp;
use crate::app::constants::CARD_CORNER_RADIUS;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::{danger_preview_delete_button, preview_overlay_configure_button};
use titan_egui_widgets::preview_overlay_action_bar_rects_three;

const PREVIEW_HOVER_MASK_A: u8 = 100;
const PREVIEW_CFG_BTN_PAD: f32 = 8.0;
const PREVIEW_OVERLAY_BTN_H: f32 = 30.0;
const PREVIEW_OVERLAY_BTN_GAP: f32 = 8.0;

pub(super) fn paint_device_preview_slot(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    card_index: usize,
    preview_key: &str,
    card_w: f32,
    lang: UiLang,
    online: bool,
) {
    let preview_h = (card_w * 9.0 / 16.0).clamp(100.0, 200.0);
    let (preview_rect, _) = ui.allocate_exact_size(Vec2::new(card_w, preview_h), Sense::empty());
    let corners = preview_slot_top_corners();
    paint_device_preview_fill(app, ui, preview_key, preview_rect, corners, lang);
    paint_preview_hover_actions(app, ui, card_index, preview_rect, corners, lang, online);
}

fn preview_slot_top_corners() -> CornerRadius {
    CornerRadius {
        nw: CARD_CORNER_RADIUS.nw,
        ne: CARD_CORNER_RADIUS.ne,
        sw: 0,
        se: 0,
    }
}

fn paint_preview_hover_actions(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    card_index: usize,
    preview_rect: Rect,
    corners: CornerRadius,
    lang: UiLang,
    online: bool,
) {
    let show_chrome = ui.rect_contains_pointer(preview_rect);
    if !show_chrome {
        return;
    }
    paint_preview_hover_mask(ui, preview_rect, corners);
    let (pow_rect, cfg_rect, del_rect) = preview_overlay_action_bar_rects_three(
        preview_rect,
        PREVIEW_CFG_BTN_PAD,
        PREVIEW_OVERLAY_BTN_GAP,
        PREVIEW_OVERLAY_BTN_H,
    );
    paint_preview_power_on_btn(ui, pow_rect, lang, online);
    paint_preview_configure_btn(ui, cfg_rect, lang, app, card_index);
    paint_preview_delete_btn(ui, del_rect, lang, app, card_index);
    ui.ctx().request_repaint();
}

fn paint_device_preview_fill(
    app: &CenterApp,
    ui: &mut egui::Ui,
    preview_key: &str,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    lang: UiLang,
) {
    if let Some(tex) = app.host_desktop_textures.get(preview_key) {
        ui.put(
            preview_rect,
            egui::Image::new(tex)
                .corner_radius(preview_corners)
                .fit_to_exact_size(preview_rect.size())
                .maintain_aspect_ratio(false),
        );
    } else {
        paint_preview_placeholder(ui, preview_rect, preview_corners, lang);
    }
}

fn paint_preview_placeholder(
    ui: &mut egui::Ui,
    preview_rect: Rect,
    preview_corners: CornerRadius,
    lang: UiLang,
) {
    ui.painter()
        .rect_filled(preview_rect, preview_corners, DEVICE_PREVIEW_PLACEHOLDER_BG);
    let note = t(lang, Msg::DeviceMgmtDesktopPreviewNote);
    let galley = WidgetText::from(
        RichText::new(note)
            .small()
            .color(DEVICE_PREVIEW_PLACEHOLDER_TEXT),
    )
    .into_galley(
        ui,
        Some(TextWrapMode::Wrap),
        preview_rect.width() * 0.92,
        TextStyle::Body,
    );
    let pos = preview_rect.center() - galley.size() * 0.5;
    ui.painter()
        .galley(pos, galley, DEVICE_PREVIEW_PLACEHOLDER_TEXT);
}

fn paint_preview_hover_mask(ui: &egui::Ui, preview_rect: Rect, preview_corners: CornerRadius) {
    ui.painter().rect_filled(
        preview_rect,
        preview_corners,
        Color32::from_black_alpha(PREVIEW_HOVER_MASK_A),
    );
}

fn paint_preview_configure_btn(
    ui: &mut egui::Ui,
    btn_rect: Rect,
    lang: UiLang,
    app: &mut CenterApp,
    card_index: usize,
) {
    if preview_overlay_configure_button(ui, btn_rect, t(lang, Msg::DeviceMgmtPreviewConfigure))
        .clicked()
    {
        app.open_host_config_from_card(card_index);
    }
}

fn paint_preview_power_on_btn(ui: &mut egui::Ui, btn_rect: Rect, lang: UiLang, online: bool) {
    if preview_overlay_configure_button(ui, btn_rect, t(lang, Msg::DeviceMgmtPreviewPowerOn))
        .clicked()
    {
        tracing::info!(online, "window preview: power-on placeholder clicked");
        ui.ctx().request_repaint();
    }
}

fn paint_preview_delete_btn(
    ui: &mut egui::Ui,
    btn_rect: Rect,
    lang: UiLang,
    app: &mut CenterApp,
    card_index: usize,
) {
    if danger_preview_delete_button(ui, btn_rect, t(lang, Msg::DeviceMgmtPreviewDelete)).clicked() {
        app.pending_delete_vm_window_row_ix = Some(card_index);
        ui.ctx().request_repaint();
    }
}
