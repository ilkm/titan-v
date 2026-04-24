//! Titan Host: egui settings + batch provision; control-plane `serve` on a background Tokio runtime.

use egui::ViewportBuilder;
use titan_host::host_app::HostApp;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([720.0, 560.0])
            .with_min_inner_size([520.0, 420.0])
            .with_title("Titan Host"),
        persistence_path: Some(std::path::PathBuf::from("titan-host")),
        // First-run window placement (Wayland 除外见 eframe 文档).
        centered: true,
        // Stale `window` RON in the same file as app JSON can restore a broken size; do not re-save it.
        persist_window: false,
        // After eframe merges storage into the viewport, force a sane size (covers persisted size 0 / bad).
        window_builder: Some(Box::new(|vb| {
            vb.with_inner_size([720.0, 560.0])
                .with_min_inner_size([520.0, 420.0])
        })),
        ..Default::default()
    };

    eframe::run_native(
        "Titan Host",
        native_options,
        Box::new(|cc| {
            titan_tray::macos_ensure_regular_activation_for_egui_app();
            titan_tray::register_host_tray_wakeup(&cc.egui_ctx);

            #[cfg(target_os = "linux")]
            titan_tray::spawn_linux_host_tray_thread();

            #[cfg(not(target_os = "linux"))]
            let initial_tray = match titan_tray::build_host_tray_icon() {
                Ok(t) => Some(t),
                Err(e) => {
                    tracing::warn!("system tray unavailable: {e}");
                    None
                }
            };
            #[cfg(target_os = "linux")]
            let initial_tray: Option<titan_tray::TrayIcon> = None;

            Ok(Box::new(HostApp::new(cc, initial_tray)))
        }),
    )
}
