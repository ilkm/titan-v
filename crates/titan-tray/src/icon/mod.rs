//! Tray bitmap: rounded brand chip + localized glyph(s).
//!
//! **Sizing.**
//! - **Windows**: notification area wants a **square** icon sized to
//!   [`GetSystemMetrics`]`(SM_CXSMICON/SM_CYSMICON)` (16×16 @ 100 % DPI, 32×32 @ 200 %, …). A
//!   non-square bitmap can be stretched or clipped by the Shell so the icon appears invisible and
//!   the hit area is wrong. See Microsoft Learn *"Notification Icons"*.
//! - **macOS**: the status bar thickness is reported by `[NSStatusBar systemStatusBar].thickness`
//!   (usually **22 pt** but **32 pt** on notch MacBook Pros). We multiply by the main screen's
//!   `backingScaleFactor` to get a square Retina-ready pixel size. The Shell scales the bitmap to
//!   fit the bar, so providing native pixels keeps the icon sharp. See Apple *NSStatusBar*.
//! - **Other (Linux panels)**: fallback 44×30 pill (legacy).
//!
//! **Colors.** The chip and glyph follow the current [`crate::TrayTheme`] so the icon blends with
//! dark taskbars/menu bars (black chip + white glyph) and light ones (white chip + black glyph).

mod draw;
mod font;
mod geom;
mod tray_pix;

use titan_common::UiLang;
use titan_i18n::{Msg, t};
use tray_icon::Icon;

use crate::menu::{self, DesktopProduct};
use crate::theme::{TrayTheme, current_tray_theme};

/// Reference non-(Windows/macOS) tray bitmap width (Linux pill shape).
#[cfg(not(any(windows, target_os = "macos")))]
pub const TRAY_ICON_WIDTH_PX: u32 = 44;
/// Reference non-(Windows/macOS) tray bitmap height.
#[cfg(not(any(windows, target_os = "macos")))]
pub const TRAY_ICON_HEIGHT_PX: u32 = 30;

/// Rounded-rect corner radius scaled to the short edge (≈ 1/7, min 1 px).
#[inline]
pub(crate) fn tray_corner_radius_px_for(w: u32, h: u32) -> i32 {
    (w.min(h) as i32 / 7).max(1)
}

/// Compose a tray [`Icon`] with the current OS theme + product glyph. Size is DPI-aware on
/// Windows and menu-bar-adaptive on macOS (see module docs).
pub fn tray_icon_for_lang(product: DesktopProduct, lang: UiLang) -> Icon {
    let (w, h) = tray_icon_target_size(product);
    let theme = current_tray_theme();
    let rgba = draw::compose_tray_rgba(product, lang, theme, w, h);
    Icon::from_rgba(rgba, w, h).expect("tray rgba dimensions valid")
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn tray_icon_target_size(_product: DesktopProduct) -> (u32, u32) {
    (TRAY_ICON_WIDTH_PX, TRAY_ICON_HEIGHT_PX)
}

#[cfg(windows)]
fn tray_icon_target_size(_product: DesktopProduct) -> (u32, u32) {
    let n = tray_square_px_windows();
    (n, n)
}

/// DPI-aware Windows tray side length from [`GetSystemMetrics`] / [`SM_CXSMICON`].
/// Shell may still scale slightly, but this value matches what Explorer uses today.
#[cfg(windows)]
fn tray_square_px_windows() -> u32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON, SM_CYSMICON};
    let cx = unsafe { GetSystemMetrics(SM_CXSMICON) };
    let cy = unsafe { GetSystemMetrics(SM_CYSMICON) };
    let n = cx.max(cy);
    if n > 0 { n as u32 } else { 16 }
}

#[cfg(target_os = "macos")]
fn tray_icon_target_size(_product: DesktopProduct) -> (u32, u32) {
    let n = macos_status_bar_pixels().unwrap_or(44);
    (n, n)
}

/// Menu-bar thickness (pt) × main-screen backing scale (px / pt) → Retina pixel side length.
/// Returns `None` if the AppKit classes cannot be resolved (e.g. called off the main thread
/// before `NSApplication.sharedApplication`).
#[cfg(target_os = "macos")]
fn macos_status_bar_pixels() -> Option<u32> {
    let thickness = macos_status_bar_thickness_pt()?;
    let scale = macos_main_screen_scale().unwrap_or(1.0);
    let px = (thickness * scale).round();
    if !px.is_finite() || px < 16.0 {
        return None;
    }
    Some(px as u32)
}

/// `[NSStatusBar systemStatusBar].thickness` in points.
#[cfg(target_os = "macos")]
fn macos_status_bar_thickness_pt() -> Option<f64> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    let cls = crate::theme::class_by_name("NSStatusBar")?;
    let thickness = unsafe {
        let bar: *mut AnyObject = msg_send![cls, systemStatusBar];
        if bar.is_null() {
            return None;
        }
        let t: f64 = msg_send![bar, thickness];
        t
    };
    (thickness.is_finite() && thickness > 0.0).then_some(thickness)
}

/// `[NSScreen mainScreen].backingScaleFactor` (typically 1.0 or 2.0 / 3.0 on Retina).
#[cfg(target_os = "macos")]
fn macos_main_screen_scale() -> Option<f64> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    let cls = crate::theme::class_by_name("NSScreen")?;
    let scale = unsafe {
        let main: *mut AnyObject = msg_send![cls, mainScreen];
        if main.is_null() {
            return None;
        }
        let s: f64 = msg_send![main, backingScaleFactor];
        s
    };
    (scale.is_finite() && scale > 0.0).then_some(scale)
}

/// Refreshes tray **icon**, **context menu**, and **tooltip** for [`UiLang`] using the **current**
/// OS tray theme (see [`TrayTheme`]). Call this whenever either the UI language or the OS theme
/// changes; [`sync_tray_if_needed`] is a convenience wrapper that detects both.
pub fn refresh_tray_icon(
    tray: &tray_icon::TrayIcon,
    product: DesktopProduct,
    lang: UiLang,
) -> tray_icon::Result<()> {
    let m = menu::build_tray_menu(product, lang).map_err(map_tray_menu_err)?;
    tray.set_menu(Some(Box::new(m)));
    tray.set_tooltip(Some(tray_tooltip(product, lang)))?;
    tray.set_icon(Some(tray_icon_for_lang(product, lang)))
}

/// Per-process cache of the last `(UiLang, TrayTheme)` pair applied to the tray icon. High byte
/// is `UiLang as u8`, low byte is `TrayTheme as u8`; the sentinel `u16::MAX` means "unset yet".
static LAST_APPLIED_TRAY_STATE: std::sync::atomic::AtomicU16 =
    std::sync::atomic::AtomicU16::new(u16::MAX);

#[inline]
fn pack_tray_state(lang: UiLang, theme: TrayTheme) -> u16 {
    ((lang as u16) << 8) | (theme as u16)
}

/// Rebuilds the tray icon when the UI language **or** the OS color theme has changed since the
/// last call. Cheap when nothing changed (one atomic load + cheap OS theme query), so it is
/// safe to call once per UI frame. Returns `true` if the icon was refreshed this call.
pub fn sync_tray_if_needed(
    tray: &tray_icon::TrayIcon,
    product: DesktopProduct,
    lang: UiLang,
) -> bool {
    use std::sync::atomic::Ordering;
    let theme = current_tray_theme();
    let packed = pack_tray_state(lang, theme);
    if LAST_APPLIED_TRAY_STATE.swap(packed, Ordering::AcqRel) == packed {
        return false;
    }
    if let Err(e) = refresh_tray_icon(tray, product, lang) {
        tracing::warn!("tray refresh: {e}");
        LAST_APPLIED_TRAY_STATE.store(u16::MAX, Ordering::Release);
        return false;
    }
    true
}

fn map_tray_menu_err(e: tray_icon::menu::Error) -> tray_icon::Error {
    tray_icon::Error::OsError(std::io::Error::other(format!("tray menu: {e}")))
}

fn tray_tooltip(product: DesktopProduct, lang: UiLang) -> &'static str {
    match product {
        DesktopProduct::Center => t(lang, Msg::BrandTitle),
        DesktopProduct::Host => t(lang, Msg::HpWinTitle),
    }
}
