//! Apple host VM backend (**placeholder**): product path targets `Virtualization.framework`; lower-level
//! **Hypervisor.framework** / QEMU `hvf` may follow for finer VM-exit control.
//!
//! [`crate::ReadMemory`] is not expected to map 1:1 to the Windows Hyper-V paravisor path; keep
//! capabilities explicit at the host/center protocol layer.

/// Capability matrix vs Hyper-V (Phase 8).
#[must_use]
pub const fn capability_matrix_vs_hyperv() -> &'static str {
    "Apple host (hvf module): power/read_memory/input not implemented; no WinHv-style guest RAM read."
}

use titan_common::{Error, Result};

use crate::traits::{InjectInput, PowerControl, ReadMemory};

/// Placeholder handle for a future Apple virtualization–backed runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct HvfBackend;

impl ReadMemory for HvfBackend {
    fn read_guest_u64(&self, _vm_id: &str, _guest_addr: u64) -> Result<u64> {
        Err(Error::NotImplemented {
            feature: "HVF guest memory read (no stable paravisor API like WinHv)",
        })
    }
}

impl InjectInput for HvfBackend {
    fn inject_mouse_move(&self, _vm_id: &str, _x: u32, _y: u32) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "HVF synthetic input / guest channel",
        })
    }
}

impl PowerControl for HvfBackend {
    fn start(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "HVF VM start (Virtualization.framework path pending)",
        })
    }

    fn stop(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "HVF VM stop (Virtualization.framework path pending)",
        })
    }
}

#[cfg(test)]
mod tests {
    use titan_common::Error;

    use super::*;

    #[test]
    fn read_memory_is_stub() {
        let err = HvfBackend.read_guest_u64("vm-1", 0).unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }

    #[test]
    fn capability_matrix_is_documented() {
        assert!(!capability_matrix_vs_hyperv().is_empty());
    }
}
