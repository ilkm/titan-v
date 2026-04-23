//! Core VMM capabilities shared across Hyper-V, KVM, and future macOS backends.

use titan_common::Result;

/// Guest physical / virtual memory reads (hypervisor-assisted).
///
/// **Reality check (Windows / Hyper-V)**: ring-3 callers generally **cannot** safely read
/// arbitrary guest RAM without a paravisor API, a kernel driver, or a cooperating guest agent.
/// Titan-v returns [`titan_common::Error::NotImplemented`] until a supported path is wired;
/// product documentation should call out EULA / lawful-use constraints for any memory access.
pub trait ReadMemory: Send + Sync {
    /// Reads an 8-byte value from the guest at `guest_addr` for `vm_id`.
    fn read_guest_u64(&self, vm_id: &str, guest_addr: u64) -> Result<u64>;
}

/// Injects HID / pointer events into the guest without in-guest agents where possible.
pub trait InjectInput: Send + Sync {
    fn inject_mouse_move(&self, vm_id: &str, x: u32, y: u32) -> Result<()>;
}

/// VM power lifecycle (start/stop/pause) for orchestration layers.
pub trait PowerControl: Send + Sync {
    fn start(&self, vm_id: &str) -> Result<()>;
    fn stop(&self, vm_id: &str) -> Result<()>;
}
