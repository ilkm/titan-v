//! Tray bitmap: rounded brand chip + localized glyph(s).
//!
//! **Windows:** the notification area requires a **square** icon sized to [`SM_CXSMICON`] /
//! [`SM_CYSMICON`] (16×16 @ 100% DPI, 32×32 @ 200%, …). A non-square bitmap can be stretched or
//! clipped by the Shell so the icon appears invisible and the hit area is wrong (right-click menu
//! never appears). See Microsoft Learn *“Notification Icons”* /
//! `GetSystemMetrics(SM_CXSMICON)`.
//!
//! **macOS:** uses the legacy 44×30 pill (Pinyin *拼* / ABC *A* tile reference).

mod draw;
mod font;
mod geom;
mod tray_pix;

use titan_common::UiLang;
use titan_i18n::{Msg, t};
use tray_icon::Icon;

use crate::menu::{self, DesktopProduct};

/// Reference **non-Windows** tray bitmap width (macOS pill shape).
#[cfg(not(windows))]
pub const TRAY_ICON_WIDTH_PX: u32 = 44;
/// Reference **non-Windows** tray bitmap height.
#[cfg(not(windows))]
pub const TRAY_ICON_HEIGHT_PX: u32 = 30;

/// Rounded-rect corner radius scaled to the short edge (≈ 1/7, min 1 px).
#[inline]
pub(crate) fn tray_corner_radius_px_for(w: u32, h: u32) -> i32 {
    (w.min(h) as i32 / 7).max(1)
}

/// **Zh**: solid white pill + `控`/`客` as **alpha cutouts**. **En**: IME-style white ring + solid
/// white `C`/`H` (ABC “A” tile). On **Windows**, the bitmap is **square** and sized to the current
/// DPI-aware tray icon metric so it renders correctly on Win10/11 notification area.
pub fn tray_icon_for_lang(product: DesktopProduct, lang: UiLang) -> Icon {
    let (w, h) = tray_icon_target_size(product);
    let rgba = draw::compose_tray_rgba(product, lang, w, h);
    Icon::from_rgba(rgba, w, h).expect("tray rgba dimensions valid")
}

#[cfg(not(windows))]
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

/// Refreshes tray **icon**, **context menu**, and **tooltip** for [`UiLang`] (e.g. after settings change).
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

fn map_tray_menu_err(e: tray_icon::menu::Error) -> tray_icon::Error {
    tray_icon::Error::OsError(std::io::Error::other(format!("tray menu: {e}")))
}

fn tray_tooltip(product: DesktopProduct, lang: UiLang) -> &'static str {
    match product {
        DesktopProduct::Center => t(lang, Msg::BrandTitle),
        DesktopProduct::Host => t(lang, Msg::HpWinTitle),
    }
}
