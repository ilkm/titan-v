//! Column / card width math aligned with Titan Center device management masonry.

use super::constants::{DEVICE_CARD_GAP, DEVICE_CARD_MAX_WIDTH, DEVICE_CARD_MIN_WIDTH};

fn device_mgmt_raw_card_w(available: f32, gap: f32, cols: usize) -> f32 {
    if cols <= 1 {
        available
    } else {
        (available - gap * (cols - 1) as f32) / cols as f32
    }
}

fn device_mgmt_refine_cols_for_width_bounds(
    cols: &mut usize,
    card_w: &mut f32,
    available: f32,
    gap: f32,
    min_w: f32,
    max_w: f32,
) {
    while *card_w > max_w && *cols < 6 {
        *cols += 1;
        *card_w = device_mgmt_raw_card_w(available, gap, *cols);
    }
    while *card_w < min_w && *cols > 1 {
        *cols -= 1;
        *card_w = device_mgmt_raw_card_w(available, gap, *cols);
    }
}

fn device_mgmt_clamp_final_card_w(
    cols: usize,
    card_w: f32,
    available: f32,
    min_w: f32,
    max_w: f32,
) -> f32 {
    let mut w = if cols <= 1 {
        available.min(max_w).max(80.0)
    } else {
        card_w.clamp(min_w, max_w)
    };
    w = w.min(max_w);
    w
}

#[must_use]
pub(crate) fn device_mgmt_cols_and_card_width(available: f32) -> (usize, f32) {
    let gap = DEVICE_CARD_GAP;
    let min_w = DEVICE_CARD_MIN_WIDTH;
    let max_w = DEVICE_CARD_MAX_WIDTH;
    if available <= 0.0 {
        return (1, 100.0);
    }
    let max_cols_fit = ((available + gap) / (min_w + gap)).floor() as usize;
    let mut cols = max_cols_fit.clamp(1, 6);
    let mut card_w = device_mgmt_raw_card_w(available, gap, cols);
    device_mgmt_refine_cols_for_width_bounds(&mut cols, &mut card_w, available, gap, min_w, max_w);
    let card_w = device_mgmt_clamp_final_card_w(cols, card_w, available, min_w, max_w);
    (cols, card_w)
}

#[must_use]
pub(crate) fn device_mgmt_card_height_hint(card_w: f32) -> f32 {
    let preview_h = (card_w * 9.0 / 16.0).clamp(100.0, 200.0);
    preview_h + 200.0
}
