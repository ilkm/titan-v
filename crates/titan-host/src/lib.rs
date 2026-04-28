//! Host node library: control-plane server and VM orchestration (binary is `titan-host`).

// `unsafe` is confined to `desktop_snapshot_win` (Windows GDI); that module opts in locally.
#![deny(unsafe_code)]

pub mod agent_bindings;
pub mod batch;
pub mod capture;
pub mod config;
pub mod desktop_snapshot;
pub mod driver_bridge;
pub mod host_app;
pub mod host_device_id;
mod host_font;
pub mod host_resources;
pub mod host_runtime_probes;
pub mod orchestrator;
pub mod tcp_tune;
pub mod ui_persist;
pub mod driver {
    pub use titan_driver::*;
}
/// Offline mother-disk tooling (Phase 2B); consumed only by the host stack.
pub mod offline_spoof {
    pub use titan_offline_spoof::*;
}
pub mod runtime;
pub mod serve;
pub use serve::ServeState;
pub mod windivert;
/// VM / hypervisor automation surface used by the host (Hyper-V on Windows, stubs elsewhere).
pub mod vmm {
    pub use titan_vmm::*;
}
/// Lua script engine surface used by the host orchestration path.
pub mod scripts {
    pub use titan_scripts::*;
}
/// Shared egui UI primitives (aligned with Titan Center).
pub use titan_egui_widgets;
/// Shared UI strings (EN/ZH) for Center and Host.
pub use titan_i18n;
