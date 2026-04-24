use crate::menu::DesktopProduct;
use tray_icon::Icon;

const TRAY_ICON_PX: u32 = 32;

pub fn tray_icon_for(product: DesktopProduct) -> Icon {
    let (r, g, b) = match product {
        DesktopProduct::Center => (37u8, 99u8, 235u8),
        DesktopProduct::Host => (5u8, 150u8, 105u8),
    };
    let mut rgba = vec![0u8; (TRAY_ICON_PX * TRAY_ICON_PX * 4) as usize];
    for px in rgba.chunks_exact_mut(4) {
        px[0] = r;
        px[1] = g;
        px[2] = b;
        px[3] = 255;
    }
    Icon::from_rgba(rgba, TRAY_ICON_PX, TRAY_ICON_PX).expect("tray icon rgba dimensions are valid")
}
