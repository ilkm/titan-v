//! Titan-v center manager: egui shell + control-plane client (M2 Ping).

mod app;

use app::CenterApp;
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
        persistence_path: Some(std::path::PathBuf::from("titan-center")),
        ..Default::default()
    };

    eframe::run_native(
        "Titan Center",
        native_options,
        Box::new(|cc| Ok(Box::new(CenterApp::new(cc)))),
    )
}
