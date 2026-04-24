//! Host node library: control-plane server and VM orchestration (binary is `titan-host`).

#![forbid(unsafe_code)]

pub mod agent_bindings;
pub mod batch;
pub mod capture;
pub mod config;
pub mod desktop_snapshot;
pub mod driver_bridge;
pub mod host_app;
mod host_font;
pub mod host_device_id;
pub mod host_resources;
pub mod host_runtime_probes;
pub mod orchestrator;
pub mod tcp_tune;
pub mod driver {
    pub use titan_driver::*;
}
pub mod runtime;
pub mod serve;
pub use serve::ServeState;
pub mod windivert;
