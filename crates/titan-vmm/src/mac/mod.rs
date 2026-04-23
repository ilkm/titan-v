//! Apple `Virtualization.framework` backend (macOS placeholder).
//!
//! Planned implementation: **Apple `Virtualization.framework`** for VM lifecycle (Linux and,
//! where supported, Windows on ARM guests), with optional later integration of
//! **`Hypervisor.framework`** / QEMU (`hvf`) if you need lower-level control.
//!
//! [`crate::ReadMemory`] is not expected to map 1:1 to the Windows Hyper-V paravisor path; keep
//! capabilities explicit at the host/center protocol layer.

/// Capability matrix vs Hyper-V (Phase 8).
#[must_use]
pub const fn capability_matrix_vs_hyperv() -> &'static str {
    "macOS Virtualization.framework path: power/read_memory/input not implemented; no WinHv-style guest RAM read."
}

use titan_common::{Error, Result};

use crate::traits::{InjectInput, PowerControl, ReadMemory};

/// Placeholder handle for a future `Virtualization.framework`–backed runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct MacBackend;

impl ReadMemory for MacBackend {
    fn read_guest_u64(&self, _vm_id: &str, _guest_addr: u64) -> Result<u64> {
        Err(Error::NotImplemented {
            feature: "Mac guest memory read (no stable paravisor API like WinHv)",
        })
    }
}

impl InjectInput for MacBackend {
    fn inject_mouse_move(&self, _vm_id: &str, _x: u32, _y: u32) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "Mac synthetic input / guest channel",
        })
    }
}

impl PowerControl for MacBackend {
    fn start(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "Mac Virtualization.framework VM start",
        })
    }

    fn stop(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "Mac Virtualization.framework VM stop",
        })
    }
}

#[cfg(test)]
mod tests {
    use titan_common::Error;

    use super::*;

    #[test]
    fn read_memory_is_stub() {
        let err = MacBackend.read_guest_u64("vm-1", 0).unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }

    #[test]
    fn capability_matrix_is_documented() {
        assert!(!capability_matrix_vs_hyperv().is_empty());
    }
}
