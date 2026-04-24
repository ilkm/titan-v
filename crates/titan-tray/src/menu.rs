//! Context menu: macOS root [`muda::Menu`] may only contain [`muda::Submenu`] children.

use tray_icon::menu::{Menu, MenuId, MenuItem, Submenu};

/// Stable menu item ids (avoid clashes if multiple Titan apps run).
pub const MENU_CENTER_SHOW: &str = "titan.center.show";
pub const MENU_CENTER_QUIT: &str = "titan.center.quit";
pub const MENU_HOST_SHOW: &str = "titan.host.show";
pub const MENU_HOST_QUIT: &str = "titan.host.quit";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopProduct {
    Center,
    Host,
}

impl DesktopProduct {
    fn submenu_title(self) -> &'static str {
        match self {
            DesktopProduct::Center => "Titan Center",
            DesktopProduct::Host => "Titan Host",
        }
    }

    fn include_show(self) -> bool {
        matches!(self, DesktopProduct::Center | DesktopProduct::Host)
    }

    pub fn quit_menu_id(self) -> MenuId {
        match self {
            DesktopProduct::Center => MenuId::new(MENU_CENTER_QUIT),
            DesktopProduct::Host => MenuId::new(MENU_HOST_QUIT),
        }
    }

    /// Tray menu id for **显示主窗口** (egui apps).
    #[must_use]
    pub fn show_menu_id(self) -> MenuId {
        match self {
            DesktopProduct::Center => MenuId::new(MENU_CENTER_SHOW),
            DesktopProduct::Host => MenuId::new(MENU_HOST_SHOW),
        }
    }
}

pub fn build_tray_menu(product: DesktopProduct) -> tray_icon::menu::Result<Menu> {
    let menu = Menu::new();
    let quit = MenuItem::with_id(product.quit_menu_id(), "退出", true, None);

    #[cfg(target_os = "macos")]
    {
        if product.include_show() {
            let show = MenuItem::with_id(product.show_menu_id(), "显示主窗口", true, None);
            let submenu = Submenu::with_items(product.submenu_title(), true, &[&show, &quit])?;
            menu.append(&submenu)?;
        } else {
            let submenu = Submenu::with_items(product.submenu_title(), true, &[&quit])?;
            menu.append(&submenu)?;
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if product.include_show() {
            let show = MenuItem::with_id(product.show_menu_id(), "显示主窗口", true, None);
            menu.append(&show)?;
        }
        menu.append(&quit)?;
    }

    Ok(menu)
}
