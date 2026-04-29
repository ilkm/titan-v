//! egui shell: control-plane settings and **窗口管理** (create-window form; VMM wiring pending).
//! VM lifecycle targets **OpenVMM** integration; prior in-tree VMM crates removed. Control TCP still speaks [`titan_common`] wire types.

mod constants;
mod model;
mod panels;
mod shell;
mod ui;
mod vm_window_device_card_clone;
mod vm_window_grid_metrics;

pub use model::{HostApp, HostUiPersist, PERSIST_KEY};
