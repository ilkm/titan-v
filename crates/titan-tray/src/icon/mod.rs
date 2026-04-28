//! Tray bitmap: rounded brand chip + localized glyph(s).

mod draw;
mod font;
mod geom;
mod tray_pix;

use titan_common::UiLang;
use titan_i18n::{t, Msg};
use tray_icon::Icon;

use crate::menu::{self, DesktopProduct};

/// Tray bitmap width (horizontal “pill”; matches macOS Pinyin **拼** / ABC **A** tile ≈44×30 reference).
pub const TRAY_ICON_WIDTH_PX: u32 = 44;
/// Tray bitmap height (same reference assets as width).
pub const TRAY_ICON_HEIGHT_PX: u32 = 30;

/// Uniform corner radius (px) for the tray rounded rectangle (Zh chip, En ring outer, and clip geometry).
pub const TRAY_CORNER_RADIUS_PX: i32 = 4;

/// Rounded-rect corner radius for tray bitmaps ([`TRAY_CORNER_RADIUS_PX`]).
#[inline]
pub fn tray_corner_radius_px() -> i32 {
    TRAY_CORNER_RADIUS_PX
}

/// **Zh**: solid white pill + `控`/`客` as **alpha cutouts** (menu bar through glyphs). **En**: IME-style **white ring** + transparent interior + solid white `C`/`H` (ABC “A” tile class). Font inset accounts for ring stroke on English.
pub fn tray_icon_for_lang(product: DesktopProduct, lang: UiLang) -> Icon {
    let rgba = draw::compose_tray_rgba(product, lang);
    Icon::from_rgba(rgba, TRAY_ICON_WIDTH_PX, TRAY_ICON_HEIGHT_PX)
        .expect("tray rgba dimensions valid")
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
