//! System tray helpers shared by **Titan Center** (egui) and **Titan Host** (`serve`).
//!
//! - Feature **`egui`**: build tray + poll/hide-window hooks for `eframe`.
//! - Feature **`tokio`**: tray "Quit" signals a [`tokio::sync::watch::Sender`] for long-running serve.
//!
//! **Windows desktop exes** must embed an application manifest that declares **ComCtl32 v6**
//! (`Microsoft.Windows.Common-Controls`); see `assets/windows/titan-desktop.manifest` and Microsoft Learn
//! *Enabling Visual Styles*. Titan’s `titan-center` / `titan-host` `build.rs` embeds it via `winres`.

pub use tray_icon::TrayIcon;

#[cfg(feature = "egui")]
mod egui;
mod icon;
mod menu;
#[cfg(feature = "tokio")]
mod serve;
#[cfg(all(feature = "tokio", windows))]
mod serve_win;

pub use icon::{refresh_tray_icon, tray_icon_for_lang};
pub use menu::{DesktopProduct, build_tray_menu};

#[cfg(feature = "egui")]
pub use egui::{
    build_host_tray_icon, build_tray_icon, hide_main_window_to_tray,
    macos_ensure_regular_activation_for_egui_app, poll_tray_for_egui, poll_tray_for_egui_product,
    register_center_tray_wakeup, register_host_tray_wakeup,
};

#[cfg(all(feature = "egui", windows))]
pub use egui::{consume_windows_tray_quit_close_request, set_windows_tray_wake_hwnd};

#[cfg(feature = "tokio")]
pub use serve::spawn_tray_shutdown_for_serve;
