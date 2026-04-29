//! Primary-display capture for [`titan_common::ControlRequest::HostDesktopSnapshot`] (JPEG).
//!
//! Production **`titan-host`** targets Windows; this module may still compile on other OSes for
//! workspace checks. Capture errors surface as plain strings.
//!
//! The **primary** display is chosen explicitly (`DisplayInfo::is_primary`), not the first entry
//! from enumeration. On Windows, capture uses GDI `SRCCOPY | CAPTUREBLT` so layered windows match
//! the visually top desktop (see `desktop_snapshot_win`).

use image::ImageEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, RgbaImage};

#[cfg(windows)]
#[path = "desktop_snapshot_win.rs"]
mod desktop_snapshot_win;

#[cfg(windows)]
fn capture_primary_rgba_unscaled() -> Result<RgbaImage, String> {
    desktop_snapshot_win::capture_primary_display_rgba()
}

#[cfg(not(windows))]
fn capture_primary_rgba_unscaled() -> Result<RgbaImage, String> {
    use screenshots::{Screen, display_info::DisplayInfo};
    let displays = DisplayInfo::all().map_err(|e| e.to_string())?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "no displays found".to_string())?;
    let shot = Screen::new(primary).capture().map_err(|e| e.to_string())?;
    let w = shot.width();
    let h = shot.height();
    let raw = shot.into_raw();
    RgbaImage::from_raw(w, h, raw).ok_or_else(|| "invalid capture buffer".to_string())
}

/// Encode a downscaled RGBA buffer as baseline JPEG (`image` crate JPEG does not accept `Rgba8`).
pub(crate) fn rgba_image_to_jpeg(
    rgba: &RgbaImage,
    jpeg_quality: u8,
) -> Result<(Vec<u8>, u32, u32), String> {
    let q = jpeg_quality.clamp(1, 95);
    let rgb = DynamicImage::from(rgba.clone()).to_rgb8();
    let tw = rgb.width();
    let th = rgb.height();
    let mut out = Vec::new();
    let enc = JpegEncoder::new_with_quality(&mut out, q);
    enc.write_image(rgb.as_raw(), tw, th, ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok((out, tw, th))
}

/// Returns JPEG bytes and encoded dimensions after downscale (preserving aspect).
pub fn capture_primary_display_jpeg(
    max_width: u32,
    max_height: u32,
    jpeg_quality: u8,
) -> Result<(Vec<u8>, u32, u32), String> {
    let shot = capture_primary_rgba_unscaled()?;
    let w = shot.width();
    let h = shot.height();
    let mut img = shot;

    let (tw, th) = thumbnail_dims(w, h, max_width.max(1), max_height.max(1));
    if tw != w || th != h {
        img = image::imageops::resize(&img, tw, th, image::imageops::FilterType::Triangle);
    }

    let (out, tw, th) = rgba_image_to_jpeg(&img, jpeg_quality)?;
    Ok((out, tw, th))
}

fn thumbnail_dims(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if w == 0 || h == 0 {
        return (1, 1);
    }
    let sx = max_w as f64 / w as f64;
    let sy = max_h as f64 / h as f64;
    let s = sx.min(sy).min(1.0);
    let nw = ((w as f64) * s).round().max(1.0) as u32;
    let nh = ((h as f64) * s).round().max(1.0) as u32;
    (nw, nh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn rgba_to_jpeg_matches_center_decode() {
        let img = RgbaImage::from_pixel(16, 16, Rgba([10u8, 40u8, 80u8, 255u8]));
        let (jpeg, w, h) = rgba_image_to_jpeg(&img, 80).expect("jpeg encode");
        assert_eq!((w, h), (16, 16));
        assert!(jpeg.len() > 32, "jpeg should have non-trivial size");
        let decoded = image::load_from_memory(&jpeg).expect("center-style decode");
        assert_eq!(decoded.width(), 16);
        assert_eq!(decoded.height(), 16);
    }
}
