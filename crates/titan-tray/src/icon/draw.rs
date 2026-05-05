//! Compose tray RGBA bitmaps that follow the active [`crate::TrayTheme`]:
//!
//! - **Windows**: rounded chip (`theme.chip_color()`) + solid glyph (`theme.glyph_color()`). High
//!   contrast, adapts to Win10/11 light ↔ dark notification area.
//! - **macOS / Linux**: legacy distinctive design — **Zh** paints a chip and erodes the glyph out
//!   of it (letter shows menu bar background); **En** paints an IME-style ring with a solid letter
//!   inside. Both ring and letter share the theme chip color so the icon auto-flips on dark ↔
//!   light menu bars.

use fontdue::Font;
use titan_common::UiLang;

use crate::menu::DesktopProduct;
use crate::theme::TrayTheme;

use super::font;
use super::geom;
use super::tray_corner_radius_px_for;
use super::tray_pix::{InnerClip, TrayPix};

/// Padding from the bitmap edge when fitting text, as a fraction of the short edge.
const TRAY_TEXT_PAD_FRAC: f32 = 0.10;
/// English IME ring stroke as a fraction of the short edge (min 1 px, max 2 px).
const TRAY_EN_RING_STROKE_FRAC: f32 = 0.07;
/// Shrink chosen font size vs max fit, as a fraction of the short edge (≈ 5%).
const TRAY_LABEL_PX_LESS_FRAC: f32 = 0.05;
/// Minimum rasterized label size as a fraction of the short edge.
const TRAY_LABEL_PX_FLOOR_FRAC: f32 = 0.35;

pub(crate) fn compose_tray_rgba(
    product: DesktopProduct,
    lang: UiLang,
    theme: TrayTheme,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let wu = w as usize;
    let hu = h as usize;
    let rad = tray_corner_radius_px_for(w, h);
    let mut rgba = vec![0u8; wu * hu * 4];
    compose_tray_body(&mut rgba, wu, hu, rad, product, lang, theme);
    rgba
}

#[cfg(windows)]
fn compose_tray_body(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    product: DesktopProduct,
    lang: UiLang,
    theme: TrayTheme,
) {
    fill_chip_rgba(rgba, w, h, rad, theme.chip_color());
    let Some(font) = font::tray_font() else {
        return;
    };
    paint_label_solid_on_chip(rgba, w, h, rad, font, product, lang, theme.glyph_color());
}

#[cfg(not(windows))]
fn compose_tray_body(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    product: DesktopProduct,
    lang: UiLang,
    theme: TrayTheme,
) {
    if lang == UiLang::Zh {
        fill_chip_rgba(rgba, w, h, rad, theme.chip_color());
    }
    if let Some(font) = font::tray_font() {
        paint_label(TrayLabelPaint {
            buf: rgba,
            w,
            h,
            rad,
            font,
            product,
            lang,
            theme,
        });
    }
}

#[cfg(not(windows))]
struct TrayLabelPaint<'a> {
    buf: &'a mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    font: &'a Font,
    product: DesktopProduct,
    lang: UiLang,
    theme: TrayTheme,
}

#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
fn paint_label_solid_on_chip(
    buf: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    font: &Font,
    product: DesktopProduct,
    lang: UiLang,
    color: [u8; 3],
) {
    let label = tray_label(product, lang);
    let wi = w as i32;
    let hi = h as i32;
    let Some(lay) = layout_tray_label(font, label, wi, hi, lang) else {
        return;
    };
    let clip = InnerClip {
        ox: 0,
        oy: 0,
        w: wi,
        h: hi,
        rad,
    };
    let mut layer = TrayPix { buf, w: wi, h: hi };
    layer.paint_string_solid(font, label, lay.px, lay.x0, lay.base, color, &clip);
}

#[inline]
fn short_edge(w: i32, h: i32) -> f32 {
    w.min(h) as f32
}

#[inline]
fn tray_en_ring_stroke_px(w: i32, h: i32) -> i32 {
    ((short_edge(w, h) * TRAY_EN_RING_STROKE_FRAC).round() as i32).clamp(1, 2)
}

fn fill_chip_rgba(rgba: &mut [u8], w: usize, h: usize, rad: i32, color: [u8; 4]) {
    let wi = w as i32;
    let hi = h as i32;
    for y in 0..hi {
        for x in 0..wi {
            if !geom::inside_round_rect(x, y, wi, hi, rad) {
                continue;
            }
            let i = ((y * wi + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(not(windows))]
fn fill_round_rect_ring(
    rgba: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    stroke: i32,
    color: [u8; 4],
) {
    let wi = w as i32;
    let hi = h as i32;
    let Some((ox, oy, iw, ih, rad_i)) = inner_round_rect_params(wi, hi, stroke) else {
        fill_chip_rgba(rgba, w, h, rad, color);
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
            rgba[i..i + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(not(windows))]
fn inner_round_rect_params(wi: i32, hi: i32, stroke: i32) -> Option<(i32, i32, i32, i32, i32)> {
    let iw = wi - 2 * stroke;
    let ih = hi - 2 * stroke;
    if iw < 4 || ih < 4 {
        return None;
    }
    let rad_outer = tray_corner_radius_px_for(wi as u32, hi as u32);
    let rad_i = (rad_outer - stroke).max(1);
    Some((stroke, stroke, iw, ih, rad_i))
}

#[cfg(not(windows))]
fn paint_label<'a>(ctx: TrayLabelPaint<'a>) {
    let label = tray_label(ctx.product, ctx.lang);
    let wi = ctx.w as i32;
    let hi = ctx.h as i32;
    let Some(lay) = layout_tray_label(ctx.font, label, wi, hi, ctx.lang) else {
        return;
    };
    match ctx.lang {
        UiLang::Zh => paint_zh_cutout(ctx.buf, wi, hi, ctx.rad, ctx.font, label, &lay),
        UiLang::En => paint_en_ring_solid(
            ctx.buf, ctx.w, ctx.h, ctx.rad, ctx.font, label, &lay, ctx.theme,
        ),
    }
}

#[cfg(not(windows))]
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

#[cfg(not(windows))]
#[allow(clippy::too_many_arguments)]
fn paint_en_ring_solid(
    buf: &mut [u8],
    w: usize,
    h: usize,
    rad: i32,
    font: &Font,
    label: &str,
    lay: &LabelLayout,
    theme: TrayTheme,
) {
    let wi = w as i32;
    let hi = h as i32;
    let stroke = tray_en_ring_stroke_px(wi, hi);
    let chip_rgba = theme.chip_color();
    let letter_rgb = [chip_rgba[0], chip_rgba[1], chip_rgba[2]];
    fill_round_rect_ring(buf, w, h, rad, stroke, chip_rgba);
    let Some(clip) = inner_clip_en(wi, hi, stroke) else {
        return;
    };
    let mut layer = TrayPix {
        buf,
        w: wi,
        h: hi,
        rad,
    };
    layer.paint_string_solid(font, label, lay.px, lay.x0, lay.base, letter_rgb, &clip);
}

#[cfg(not(windows))]
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
        UiLang::En => tray_en_ring_stroke_px(wi, hi) as f32,
        UiLang::Zh => 0.0,
    };
    let px_fit = pick_max_font_px(font, label, wi, hi, border_pad);
    let short = short_edge(wi, hi);
    let px_less = short * TRAY_LABEL_PX_LESS_FRAC;
    let px_floor = (short * TRAY_LABEL_PX_FLOOR_FRAC).max(6.0);
    let px = (px_fit - px_less).max(px_floor);
    let (ink_top, ink_bot) = string_ink_vertical_bounds(font, label, px)?;
    let tw = string_advance_width(font, label, px);
    let x0 = label_center_x(wi, tw);
    let base = baseline_from_ink_bounds(hi, ink_top, ink_bot);
    Some(LabelLayout { px, x0, base })
}

fn pick_max_font_px(font: &Font, label: &str, w: i32, h: i32, border_pad: f32) -> f32 {
    let pad = short_edge(w, h) * TRAY_TEXT_PAD_FRAC + border_pad;
    let max_w = (w as f32 - 2.0 * pad).max(4.0);
    let max_h = (h as f32 - 2.0 * pad).max(4.0);
    for px_int in (6..=160).rev() {
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
        if let Some(p) = prev
            && let Some(k) = font.horizontal_kern(p, c, px)
        {
            pen += k;
        }
        pen += font.metrics(c, px).advance_width;
        prev = Some(c);
    }
    pen
}

fn label_center_x(w: i32, text_w: f32) -> f32 {
    ((w as f32) - text_w) / 2.0
}
