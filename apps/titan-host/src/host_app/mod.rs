//! egui shell: control-plane settings and a placeholder **窗口管理** tab.
//! VM / Hyper-V automation crates were removed; control TCP still speaks [`titan_common`] wire types.

mod constants;
mod model;
mod panels;
mod shell;
mod ui;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
