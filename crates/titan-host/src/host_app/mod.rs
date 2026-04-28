//! egui shell: control-plane settings and a placeholder **窗口管理** tab.
//! Hyper-V batch provisioning remains in [`crate::batch`] for programmatic / TOML use.

mod chrome;
mod constants;
mod eframe_impl;
mod model;
mod new_serve;
mod panels;
mod theme;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
