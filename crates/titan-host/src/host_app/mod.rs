//! egui shell: service settings, agent bindings, and batch VM provisioning (no hand-edited TOML).

mod eframe_impl;
mod model;
mod new_serve;
mod panels;

pub use model::{AgentBindingRow, HostApp, HostUiPersist};
