//! egui shell: control-plane settings and a placeholder **窗口管理** tab.
//! VM lifecycle targets **OpenVMM** integration; prior in-tree VMM crates removed. Control TCP still speaks [`titan_common`] wire types.

mod constants;
mod model;
mod panels;
mod shell;
mod ui;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
