//! Titan Host: egui settings + batch provision; control-plane `serve` on a background Tokio runtime.

use egui::ViewportBuilder;
use titan_common::UiLang;
use titan_host::host_app::{HostApp, HostUiPersist, PERSIST_KEY};
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

fn host_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([720.0, 560.0])
            .with_min_inner_size([520.0, 420.0])
            .with_title("Titan Host"),
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
    }
}

fn tray_lang_from_storage(cc: &eframe::CreationContext<'_>) -> UiLang {
    cc.storage
        .and_then(|s| s.get_string(PERSIST_KEY))
        .and_then(|j| serde_json::from_str::<HostUiPersist>(&j).ok())
        .map(|p| p.ui_lang)
        .unwrap_or_default()
}

fn host_initial_tray(cc: &eframe::CreationContext<'_>) -> Option<titan_tray::TrayIcon> {
    titan_tray::macos_ensure_regular_activation_for_egui_app();
    titan_tray::register_host_tray_wakeup(&cc.egui_ctx);

    match titan_tray::build_host_tray_icon(tray_lang_from_storage(cc)) {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!("system tray unavailable: {e}");
            None
        }
    }
}

fn main() -> eframe::Result<()> {
    init_tracing();
    eframe::run_native(
        "Titan Host",
        host_native_options(),
        Box::new(|cc| Ok(Box::new(HostApp::new(cc, host_initial_tray(cc))))),
    )
}
