//! Linux KVM + virtio backend (placeholder).
//!
//! Optional **`virsh`** list / power lives in [`virsh_shell`] (libvirt shell, not full QEMU/QMP).
//! Future work: map [`crate::ReadMemory`], [`crate::InjectInput`] to `libvirt` / `kvm` ioctls /
//! virtio channels as appropriate.

pub mod virsh_shell;

/// Capability matrix vs Hyper-V (Phase 8): strings only — no runtime probe here.
#[must_use]
pub const fn capability_matrix_vs_hyperv() -> &'static str {
    "KVM: virsh-backed list/power on Linux when client tools exist; read_memory/input/streaming not implemented; parity with Hyper-V is not guaranteed."
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
    fn start(&self, vm_id: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !virsh_shell::virsh_version_available_blocking() {
                return Err(Error::VmmRejected {
                    message: "virsh is not available on PATH (install libvirt-client).".into(),
                });
            }
            virsh_shell::domain_set_power_blocking(vm_id, true)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = vm_id;
            Err(Error::NotImplemented {
                feature: "KVM domain start",
            })
        }
    }

    fn stop(&self, vm_id: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if !virsh_shell::virsh_version_available_blocking() {
                return Err(Error::VmmRejected {
                    message: "virsh is not available on PATH (install libvirt-client).".into(),
                });
            }
            virsh_shell::domain_set_power_blocking(vm_id, false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = vm_id;
            Err(Error::NotImplemented {
                feature: "KVM domain stop",
            })
        }
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

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn kvm_power_start_not_implemented_off_linux() {
        let err = KvmBackend.start("vm-1").unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }), "{err}");
    }
}
