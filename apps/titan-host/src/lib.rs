//! Host node library: egui settings + TCP control-plane server (`titan_host::serve`).

// `unsafe` is confined to `desktop_snapshot_win` (Windows GDI); that module opts in locally.
#![deny(unsafe_code)]

pub mod agent_binding_table;
pub(crate) mod debug_agent_log;
pub mod desktop_snapshot;
pub mod host_app;
pub mod host_device_id;
mod host_font;
pub mod host_paths;
pub mod host_resources;
pub mod host_runtime_probes;
pub mod ui_persist;
/// Shared egui UI primitives (aligned with Titan Center).
pub use titan_egui_widgets;
/// Shared UI strings (EN/ZH) for Center and Host.
pub use titan_i18n;
pub mod serve;
pub use serve::ServeState;
