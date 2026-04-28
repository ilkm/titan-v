//! Compose tray RGBA: Zh = solid chip + cutout text; En = white ring + solid white letter (ABC tile).

use fontdue::Font;
use titan_common::UiLang;

use crate::menu::DesktopProduct;

use super::font;
use super::geom;
use super::tray_corner_radius_px;
use super::tray_pix::{InnerClip, TrayPix};
use super::TRAY_ICON_HEIGHT_PX;
use super::TRAY_ICON_WIDTH_PX;

/// Padding from the bitmap edge when fitting text (independent of [`tray_corner_radius_px`]).
const TRAY_TEXT_PAD_PX: f32 = 3.0;
/// English IME ring stroke in pixels (white outline, interior transparent).
const TRAY_EN_RING_STROKE_PX: i32 = 2;
/// Shrink chosen font size vs max fit (“小 2 个号” ≈ −2 px).
const TRAY_LABEL_PX_LESS: f32 = 2.0;
/// Minimum rasterized label size after [`TRAY_LABEL_PX_LESS`].
const TRAY_LABEL_PX_FLOOR: f32 = 6.0;

pub(crate) fn compose_tray_rgba(product: DesktopProduct, lang: UiLang) -> Vec<u8> {
    let w = TRAY_ICON_WIDTH_PX as usize;
    let h = TRAY_ICON_HEIGHT_PX as usize;
    let rad = tray_corner_radius_px();
    let mut rgba = vec![0u8; w * h * 4];
    if lang == UiLang::Zh {
        fill_white_chip(&mut rgba, w, h, rad);
    }
    if let Some(font) = font::tray_font() {
        paint_label(&mut rgba, w, h, rad, font, product, lang);
    }
    rgba
}

fn fill_white_chip(rgba: &mut [u8], w: usize, h: usize, rad: i32) {
    let wi = w as i32;
    let hi = h as i32;
    for y in 0..hi {
        for x in 0..wi {
            if !geom::inside_round_rect(x, y, wi, hi, rad) {
                continue;
            }
            let i = ((y * wi + x) * 4) as usize;
            rgba[i] = 255;
            rgba[i + 1] = 255;
            rgba[i + 2] = 255;
            rgba[i + 3] = 255;
        }
    }
}

fn fill_white_round_rect_ring(rgba: &mut [u8], w: usize, h: usize, rad: i32, stroke: i32) {
    let wi = w as i32;
    let hi = h as i32;
    let Some((ox, oy, iw, ih, rad_i)) = inner_round_rect_params(wi, hi, stroke) else {
        fill_white_chip(rgba, w, h, rad);
        return;
    };
    for y in 0..hi {
        for x in 0..wi {
            let outer = geom::inside_round_rect(x, y, wi, hi, rad);
            let inner = geom::inside_round_rect(x - ox, y - oy, iw, ih, rad_i);
            if !outer || inner {
                continue;
            }
            let i = ((y * wi + x) * 4) as usize;
            rgba[i] = 255;
            rgba[i + 1] = 255;
            rgba[i + 2] = 255;
            rgba[i + 3] = 255;
        }
    }
}

fn inner_round_rect_params(wi: i32, hi: i32, stroke: i32) -> Option<(i32, i32, i32, i32, i32)> {
    let iw = wi - 2 * stroke;
    let ih = hi - 2 * stroke;
    if iw < 4 || ih < 4 {
        return None;
    }
    let rad_i = (tray_corner_radius_px() - stroke).max(1);
    Some((stroke, stroke, iw, ih, rad_i))
}

fn paint_label(
    buf: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    font: &Font,
    product: DesktopProduct,
    lang: UiLang,
) {
    let label = tray_label(product, lang);
    let wi = w as i32;
    let hi = h as i32;
    let Some(lay) = layout_tray_label(font, label, wi, hi, lang) else {
        return;
    };
    match lang {
        UiLang::Zh => paint_zh_cutout(buf, wi, hi, rad, font, label, &lay),
        UiLang::En => paint_en_ring_solid(buf, w, h, rad, font, label, &lay),
    }
}

fn paint_zh_cutout(
    buf: &mut [u8],
    wi: i32,
    hi: i32,
    rad: i32,
    font: &Font,
    label: &str,
    lay: &LabelLayout,
) {
    let mut layer = TrayPix {
        buf,
        w: wi,
        h: hi,
        rad,
    };
    layer.paint_string(font, label, lay.px, lay.x0, lay.base);
}

fn paint_en_ring_solid(
    buf: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    font: &Font,
    label: &str,
    lay: &LabelLayout,
) {
    let wi = w as i32;
    let hi = h as i32;
    let stroke = TRAY_EN_RING_STROKE_PX;
    fill_white_round_rect_ring(buf, w, h, rad, stroke);
    let Some(clip) = inner_clip_en(wi, hi, stroke) else {
        return;
    };
    let mut layer = TrayPix {
        buf,
        w: wi,
        h: hi,
        rad,
    };
    layer.paint_string_solid(font, label, lay.px, lay.x0, lay.base, &clip);
}

fn inner_clip_en(wi: i32, hi: i32, stroke: i32) -> Option<InnerClip> {
    let (ox, oy, iw, ih, rad_i) = inner_round_rect_params(wi, hi, stroke)?;
    Some(InnerClip {
        ox,
        oy,
        w: iw,
        h: ih,
        rad: rad_i,
    })
}

struct LabelLayout {
    px: f32,
    x0: f32,
    base: f32,
}

fn layout_tray_label(
    font: &Font,
    label: &str,
    wi: i32,
    hi: i32,
    lang: UiLang,
) -> Option<LabelLayout> {
    let border_pad = match lang {
        UiLang::En => TRAY_EN_RING_STROKE_PX as f32,
        UiLang::Zh => 0.0,
    };
    let px_fit = pick_max_font_px(font, label, wi, hi, border_pad);
    let px = (px_fit - TRAY_LABEL_PX_LESS).max(TRAY_LABEL_PX_FLOOR);
    let (ink_top, ink_bot) = string_ink_vertical_bounds(font, label, px)?;
    let tw = string_advance_width(font, label, px);
    let x0 = label_center_x(wi, tw);
    let base = baseline_from_ink_bounds(hi, ink_top, ink_bot);
    Some(LabelLayout { px, x0, base })
}

fn pick_max_font_px(font: &Font, label: &str, w: i32, h: i32, border_pad: f32) -> f32 {
    let pad = TRAY_TEXT_PAD_PX + border_pad;
    let max_w = (w as f32 - 2.0 * pad).max(4.0);
    let max_h = (h as f32 - 2.0 * pad).max(4.0);
    for px_int in (6..=80).rev() {
        if let Some(px) = try_font_px(font, label, px_int, max_w, max_h) {
            return px;
        }
    }
    10.0
}

fn try_font_px(font: &Font, label: &str, px_int: i32, max_w: f32, max_h: f32) -> Option<f32> {
    let px = px_int as f32;
    let (ink_top, ink_bot) = string_ink_vertical_bounds(font, label, px)?;
    let ink_h = ink_bot - ink_top;
    if ink_h > max_h {
        return None;
    }
    let tw = string_advance_width(font, label, px);
    (tw <= max_w).then_some(px)
}

fn string_ink_vertical_bounds(font: &Font, label: &str, px: f32) -> Option<(f32, f32)> {
    let mut prev: Option<char> = None;
    let mut ink_top = f32::MAX;
    let mut ink_bot = f32::NEG_INFINITY;
    for c in label.chars() {
        if let Some(p) = prev {
            let _ = font.horizontal_kern(p, c, px);
        }
        let m = font.metrics(c, px);
        let t = m.ymin as f32;
        let b = t + m.height as f32;
        ink_top = ink_top.min(t);
        ink_bot = ink_bot.max(b);
        prev = Some(c);
    }
    (ink_top.is_finite() && ink_bot.is_finite()).then_some((ink_top, ink_bot))
}

fn baseline_from_ink_bounds(h: i32, ink_top: f32, ink_bot: f32) -> f32 {
    (h as f32 - ink_top - ink_bot) / 2.0
}

fn tray_label(product: DesktopProduct, lang: UiLang) -> &'static str {
    match (product, lang) {
        (DesktopProduct::Center, UiLang::Zh) => "控",
        (DesktopProduct::Center, UiLang::En) => "C",
        (DesktopProduct::Host, UiLang::Zh) => "客",
        (DesktopProduct::Host, UiLang::En) => "H",
    }
}

fn string_advance_width(font: &Font, s: &str, px: f32) -> f32 {
    let mut prev: Option<char> = None;
    let mut pen = 0.0_f32;
    for c in s.chars() {
        if let Some(p) = prev {
            if let Some(k) = font.horizontal_kern(p, c, px) {
                pen += k;
            }
        }
        pen += font.metrics(c, px).advance_width;
        prev = Some(c);
    }
    pen
}

fn label_center_x(w: i32, text_w: f32) -> f32 {
    ((w as f32) - text_w) / 2.0
}
