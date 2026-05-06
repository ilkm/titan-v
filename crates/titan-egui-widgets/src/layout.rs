use egui::{Rect, pos2, vec2};

/// Returns right-aligned `(configure, delete)` button rects inside a preview area.
pub fn preview_overlay_action_bar_rects(
    preview_rect: Rect,
    pad: f32,
    gap: f32,
    button_h: f32,
) -> (Rect, Rect) {
    let y = preview_rect.bottom() - pad - button_h;
    let max_pair = (preview_rect.width() - pad * 2.0 - gap).max(0.0);
    let w_cfg = (max_pair * 0.52).clamp(56.0, 120.0);
    let w_del = (max_pair - gap - w_cfg).clamp(48.0, 100.0);
    let right_x = preview_rect.right() - pad;
    let del_min = pos2(right_x - w_del, y);
    let cfg_min = pos2(right_x - w_del - gap - w_cfg, y);
    (
        Rect::from_min_size(cfg_min, vec2(w_cfg, button_h)),
        Rect::from_min_size(del_min, vec2(w_del, button_h)),
    )
}

/// Returns right-aligned `(power_on, configure, delete)` button rects inside a preview area.
pub fn preview_overlay_action_bar_rects_three(
    preview_rect: Rect,
    pad: f32,
    gap: f32,
    button_h: f32,
) -> (Rect, Rect, Rect) {
    let y = preview_rect.bottom() - pad - button_h;
    let total = (preview_rect.width() - pad * 2.0 - gap * 2.0).max(0.0);
    let w_pow = (total * 0.32).clamp(50.0, 96.0);
    let w_cfg = (total * 0.36).clamp(56.0, 120.0);
    let w_del = (total - w_pow - w_cfg).clamp(48.0, 100.0);
    let right_x = preview_rect.right() - pad;
    let del_min = pos2(right_x - w_del, y);
    let cfg_min = pos2(right_x - w_del - gap - w_cfg, y);
    let pow_min = pos2(right_x - w_del - gap - w_cfg - gap - w_pow, y);
    (
        Rect::from_min_size(pow_min, vec2(w_pow, button_h)),
        Rect::from_min_size(cfg_min, vec2(w_cfg, button_h)),
        Rect::from_min_size(del_min, vec2(w_del, button_h)),
    )
}
