//! Host node library: control-plane server (binary is `titan-host`).

// `unsafe` is confined to `desktop_snapshot_win` (Windows GDI); that module opts in locally.
#![deny(unsafe_code)]

pub mod agent_binding_table;
pub mod agent_bindings;
pub mod control_plane;
pub mod desktop_snapshot;
pub mod driver_bridge;
pub mod host_app;
pub mod host_device_id;
mod host_font;
pub mod host_resources;
pub mod host_runtime_probes;
pub mod tcp_tune;
pub mod ui_persist;
pub mod windivert;
/// Shared egui UI primitives (aligned with Titan Center).
pub use titan_egui_widgets;
/// Shared UI strings (EN/ZH) for Center and Host.
pub use titan_i18n;
pub mod serve;
pub use serve::ServeState;
