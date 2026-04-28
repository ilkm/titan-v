//! egui shell: control-plane settings and a placeholder **窗口管理** tab.
//! VM / Hyper-V automation crates were removed; control TCP still speaks [`titan_common`] wire types.

mod chrome;
mod constants;
mod eframe_impl;
mod model;
mod new_serve;
mod panels;
mod theme;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
