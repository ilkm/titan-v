//! Pixel buffer + rounded clip for tray glyph painting (Zh cutout vs En solid).

use fontdue::Font;

use super::geom;

/// Inner rounded rect for clipping English solid glyphs (absolute bitmap coords).
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
    pub(crate) rad: i32,
}

impl TrayPix<'_> {
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

    pub(crate) fn paint_string_solid(
        &mut self,
        font: &Font,
        label: &str,
        font_px: f32,
        mut pen_x: f32,
        baseline: f32,
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
            self.blit_glyph_solid(&bitmap, pen_x, baseline, m, clip);
            pen_x += m.advance_width;
            prev = Some(ch);
        }
    }

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
        clip: &InnerClip,
    ) {
        let gx = (pen_x + m.xmin as f32).round() as i32;
        let gy = (baseline + m.ymin as f32).round() as i32;
        let gw = m.width;
        let gh = m.height;
        for row in 0..gh {
            for col in 0..gw {
                let g = bitmap[row * gw + col];
                self.blit_solid_sample(g, gx + col as i32, gy + row as i32, clip);
            }
        }
    }

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
        let px = gx + col as i32;
        let py = gy + row as i32;
        self.punch_white(px, py, g);
    }

    fn blit_solid_sample(&mut self, g: u8, px: i32, py: i32, clip: &InnerClip) {
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
        blend_white_src_over(&mut self.buf[i..i + 4], g);
    }

    /// Erode alpha where the glyph is dark: “transparent” letter on an opaque white chip.
    fn punch_white(&mut self, px: i32, py: i32, glyph_alpha: u8) {
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
        self.buf[i] = 255;
        self.buf[i + 1] = 255;
        self.buf[i + 2] = 255;
        self.buf[i + 3] = new_a.min(255) as u8;
    }
}

fn blend_white_src_over(px: &mut [u8], src_a: u8) {
    if src_a == 0 || px.len() < 4 {
        return;
    }
    let s = src_a as u32;
    let dr = px[0] as u32;
    let dg = px[1] as u32;
    let db = px[2] as u32;
    let da = px[3] as u32;
    let inv = 255u32 - s;
    let out_a = s + (da * inv + 127) / 255;
    if out_a == 0 {
        return;
    }
    px[0] = ((255 * s + dr * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[1] = ((255 * s + dg * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[2] = ((255 * s + db * da * inv / 255 + out_a / 2) / out_a).min(255) as u8;
    px[3] = out_a.min(255) as u8;
}
