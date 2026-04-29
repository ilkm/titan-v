//! Titan-v center manager: egui shell + control-plane client (Hello/Ping over framed TCP).

#![cfg_attr(windows, windows_subsystem = "windows")]

use titan_center::app::CenterApp;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([920.0, 560.0])
            .with_title("Titan Center"),
        ..Default::default()
    };

    eframe::run_native(
        "Titan Center",
        native_options,
        Box::new(|cc| {
            titan_tray::register_center_tray_wakeup(&cc.egui_ctx);
            #[cfg(windows)]
            {
                use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
                if let Ok(h) = cc.window_handle()
                    && let RawWindowHandle::Win32(w) = h.as_raw()
                {
                    titan_tray::set_windows_tray_wake_hwnd(w.hwnd.get());
                }
            }

            let app = CenterApp::new(cc);
            Ok(Box::new(app))
        }),
    )
}
