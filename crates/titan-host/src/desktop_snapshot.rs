//! Primary-display capture for [`titan_common::ControlRequest::HostDesktopSnapshot`] (JPEG).
//!
//! **macOS**: grant **Screen Recording** (and often **Accessibility**) to the `titan-host` binary
//! in *System Settings → Privacy & Security*; capture may return errors until permission is granted.

use image::codecs::jpeg::JpegEncoder;
use image::ImageEncoder;
use image::{DynamicImage, ExtendedColorType, RgbaImage};
use screenshots::Screen;

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
    let screens = Screen::all().map_err(|e| {
        #[cfg(target_os = "macos")]
        {
            format!(
                "{e} (macOS: enable Screen Recording for titan-host in System Settings → Privacy & Security)"
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            e.to_string()
        }
    })?;
    let screen = screens
        .into_iter()
        .next()
        .ok_or_else(|| "no displays found".to_string())?;
    let shot = screen.capture().map_err(|e| {
        #[cfg(target_os = "macos")]
        {
            format!(
                "{e} (macOS: Screen Recording permission required for desktop preview; check Privacy & Security)"
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            e.to_string()
        }
    })?;
    let w = shot.width();
    let h = shot.height();
    let raw = shot.into_raw();
    let mut img =
        RgbaImage::from_raw(w, h, raw).ok_or_else(|| "invalid capture buffer".to_string())?;

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
