//! Linux KVM + virtio backend (placeholder).
//!
//! Future work: map [`crate::ReadMemory`], [`crate::InjectInput`], [`crate::PowerControl`] to
//! `libvirt` / `kvm` ioctls / virtio channels as appropriate.

/// Capability matrix vs Hyper-V (Phase 8): strings only — no runtime probe here.
#[must_use]
pub const fn capability_matrix_vs_hyperv() -> &'static str {
    "KVM (planned libvirt): power/read_memory/input/streaming not implemented; parity with Hyper-V is explicitly not guaranteed."
}

use titan_common::{Error, Result};

use crate::traits::{InjectInput, PowerControl, ReadMemory};

/// Placeholder handle for a future KVM-backed runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct KvmBackend;

impl ReadMemory for KvmBackend {
    fn read_guest_u64(&self, _vm_id: &str, _guest_addr: u64) -> Result<u64> {
        Err(Error::NotImplemented {
            feature: "KVM read guest memory",
        })
    }
}

impl InjectInput for KvmBackend {
    fn inject_mouse_move(&self, _vm_id: &str, _x: u32, _y: u32) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "KVM virtio input",
        })
    }
}

impl PowerControl for KvmBackend {
    fn start(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "KVM domain start",
        })
    }

    fn stop(&self, _vm_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "KVM domain stop",
        })
    }
}

#[cfg(test)]
mod tests {
    use titan_common::Error;

    use super::*;

    #[test]
    fn read_memory_is_stub() {
        let err = KvmBackend.read_guest_u64("vm-1", 0).unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }

    #[test]
    fn capability_matrix_is_documented() {
        assert!(!capability_matrix_vs_hyperv().is_empty());
    }
}
