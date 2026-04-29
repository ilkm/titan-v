//! Independent fork of Connect **device card** visuals for window-management VM rows.
//!
//! Do not import from `ui/devices/device_card.rs`; keep this tree self-contained.

mod device_card_dup;
mod helpers_dup;

pub use device_card_dup::paint_vm_window_device_card_clone;
