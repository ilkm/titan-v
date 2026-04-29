//! Pixel buffer + rounded clip for tray glyph painting (Zh cutout vs En/Windows solid).
//!
//! Both "solid" (paint letter in arbitrary color) and "cutout" (erode chip alpha so the letter
//! shows the menu bar background through it) paths are parameter-clean so the caller can pick
//! chip / glyph colors from the active [`crate::TrayTheme`].

use fontdue::Font;

use super::geom;

/// Inner rounded rect for clipping solid glyphs (absolute bitmap coords).
pub(crate) struct InnerClip {
    pub(crate) ox: i32,
    pub(crate) oy: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
    pub(crate) rad: i32,
}

pub(crate) struct TrayPix<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) w: i32,
    pub(crate) h: i32,
    /// Round-rect corner radius for the Zh "cutout" alpha mask — only used on non-Windows (the
    /// Windows tray uses a solid chip with solid glyphs, see `icon/draw.rs`).
    #[cfg(not(windows))]
    pub(crate) rad: i32,
}

impl TrayPix<'_> {
    /// Non-Windows Zh "cutout" path: erode the chip alpha where the glyph is opaque so the
    /// letter shows the menu bar background through it. Leaves RGB as-is (chip stays themed).
    #[cfg(not(windows))]
    pub(crate) fn paint_string(
        &mut self,
        font: &Font,
        label: &str,
        font_px: f32,
        mut pen_x: f32,
        baseline: f32,
    ) {
        let mut prev: Option<char> = None;
        for ch in label.chars() {
            if let Some(p) = prev
                && let Some(k) = font.horizontal_kern(p, ch, font_px)
            {
                pen_x += k;
            }
            let (m, bitmap) = font.rasterize(ch, font_px);
            self.blit_glyph_cutout(&bitmap, pen_x, baseline, m);
            pen_x += m.advance_width;
            prev = Some(ch);
        }
    }

    /// Paint a solid-color glyph string clipped to an inner rounded rect (Windows chip, En ring).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_string_solid(
        &mut self,
        font: &Font,
        label: &str,
        font_px: f32,
        mut pen_x: f32,
        baseline: f32,
        color: [u8; 3],
        clip: &InnerClip,
    ) {
        let mut prev: Option<char> = None;
        for ch in label.chars() {
            if let Some(p) = prev
                && let Some(k) = font.horizontal_kern(p, ch, font_px)
            {
                pen_x += k;
            }
            let (m, bitmap) = font.rasterize(ch, font_px);
            self.blit_glyph_solid(&bitmap, pen_x, baseline, m, color, clip);
            pen_x += m.advance_width;
            prev = Some(ch);
        }
    }

    #[cfg(not(windows))]
    fn blit_glyph_cutout(&mut self, bitmap: &[u8], pen_x: f32, baseline: f32, m: fontdue::Metrics) {
        let gx = (pen_x + m.xmin as f32).round() as i32;
        let gy = (baseline + m.ymin as f32).round() as i32;
        let gw = m.width;
        let gh = m.height;
        for row in 0..gh {
            for col in 0..gw {
                self.blit_one_cutout(bitmap, gx, gy, row, col, gw);
            }
        }
    }

    fn blit_glyph_solid(
        &mut self,
        bitmap: &[u8],
        pen_x: f32,
        baseline: f32,
        m: fontdue::Metrics,
        color: [u8; 3],
        clip: &InnerClip,
    ) {
        let gx = (pen_x + m.xmin as f32).round() as i32;
        let gy = (baseline + m.ymin as f32).round() as i32;
        let gw = m.width;
        let gh = m.height;
        for row in 0..gh {
            for col in 0..gw {
                let g = bitmap[row * gw + col];
                self.blit_solid_sample(g, gx + col as i32, gy + row as i32, color, clip);
            }
        }
    }

    #[cfg(not(windows))]
    fn blit_one_cutout(
        &mut self,
        bitmap: &[u8],
        gx: i32,
        gy: i32,
        row: usize,
        col: usize,
        gw: usize,
    ) {
        let g = bitmap[row * gw + col];
        if g == 0 {
            return;
        }
        self.erode_alpha(gx + col as i32, gy + row as i32, g);
    }

    fn blit_solid_sample(&mut self, g: u8, px: i32, py: i32, color: [u8; 3], clip: &InnerClip) {
        if g == 0 {
            return;
        }
        if px < 0 || py < 0 || px >= self.w || py >= self.h {
            return;
        }
        if !geom::inside_round_rect(px - clip.ox, py - clip.oy, clip.w, clip.h, clip.rad) {
            return;
        }
        let i = ((py * self.w + px) * 4) as usize;
        blend_src_over(&mut self.buf[i..i + 4], color, g);
    }

    /// Erode alpha inside the chip where the glyph is opaque — the chip RGB stays so the
    /// cutout letter reveals the menu bar background behind the tray icon.
    #[cfg(not(windows))]
    fn erode_alpha(&mut self, px: i32, py: i32, glyph_alpha: u8) {
        if px < 0 || py < 0 || px >= self.w || py >= self.h {
            return;
        }
        if !geom::inside_round_rect(px, py, self.w, self.h, self.rad) {
            return;
        }
        let i = ((py * self.w + px) * 4) as usize;
        let g = glyph_alpha as u32;
        let cur = self.buf[i + 3] as u32;
        let new_a = (cur * (255 - g) + 127) / 255;
        self.buf[i + 3] = new_a.min(255) as u8;
    }
}

/// Standard non-premultiplied RGBA source-over blend with an arbitrary glyph color + alpha.
fn blend_src_over(px: &mut [u8], color: [u8; 3], src_a: u8) {
    if src_a == 0 || px.len() < 4 {
        return;
    }
    let s = src_a as u32;
    let [cr, cg, cb] = color;
    let (dr, dg, db, da) = (px[0] as u32, px[1] as u32, px[2] as u32, px[3] as u32);
    let inv = 255u32 - s;
    let out_a = s + (da * inv + 127) / 255;
    if out_a == 0 {
        return;
    }
    px[0] = ((cr as u32 * s + dr * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[1] = ((cg as u32 * s + dg * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[2] = ((cb as u32 * s + db * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[3] = out_a.min(255) as u8;
}
