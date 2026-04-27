//! System tray helpers shared by **Titan Center** (egui) and **Titan Host** (`serve`).
//!
//! - Feature **`egui`**: build tray + poll/hide-window hooks for `eframe`.
//! - Feature **`tokio`**: tray "Quit" signals a [`tokio::sync::watch::Sender`] for long-running serve.

pub use tray_icon::TrayIcon;

#[cfg(feature = "egui")]
mod egui;
mod icon;
mod menu;
#[cfg(feature = "tokio")]
mod serve;

pub use icon::tray_icon_for;
pub use menu::{build_tray_menu, DesktopProduct};

#[cfg(feature = "egui")]
pub use egui::{
    apply_close_hides_to_tray, build_host_tray_icon, build_tray_icon,
    macos_ensure_regular_activation_for_egui_app, poll_tray_for_egui, poll_tray_for_egui_product,
    register_center_tray_wakeup, register_host_tray_wakeup,
};

#[cfg(feature = "tokio")]
pub use serve::spawn_tray_shutdown_for_serve;
