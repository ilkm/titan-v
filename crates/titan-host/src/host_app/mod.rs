//! egui shell: service settings and batch VM provisioning (no hand-edited TOML).

mod chrome;
mod constants;
mod eframe_impl;
mod model;
mod new_serve;
mod panels;
mod theme;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
