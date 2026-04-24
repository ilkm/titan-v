//! Cross-OS **ListVms** and **domain power** entry points for `titan-host` (single owner per OS path).

use titan_common::{state::VmPowerState, Result};

/// Lists VMs for the control plane / telemetry. macOS returns an empty list until backed.
pub fn list_vms_blocking() -> Result<Vec<(String, VmPowerState)>> {
    #[cfg(windows)]
    {
        return crate::hyperv::list_vms_blocking();
    }
    #[cfg(target_os = "linux")]
    {
        if !crate::kvm::virsh_shell::virsh_version_available_blocking() {
            tracing::debug!("platform_vm list: virsh not on PATH; returning empty VM list");
            return Ok(Vec::new());
        }
        return crate::kvm::virsh_shell::list_domains_blocking();
    }
    #[cfg(all(not(windows), not(target_os = "linux")))]
    {
        tracing::debug!("platform_vm list: macOS placeholder; empty VM list");
        Ok(Vec::new())
    }
}

/// Starts or stops one domain / VM using the active platform backend.
pub fn domain_set_power_blocking(vm_name: &str, start: bool) -> Result<()> {
    #[cfg(windows)]
    {
        let b = crate::hyperv::HypervBackend;
        use crate::traits::PowerControl;
        return if start {
            b.start(vm_name)
        } else {
            b.stop(vm_name)
        };
    }
    #[cfg(target_os = "linux")]
    {
        return crate::kvm::virsh_shell::domain_set_power_blocking(vm_name, start);
    }
    #[cfg(all(not(windows), not(target_os = "linux")))]
    {
        let _ = (vm_name, start);
        Err(titan_common::Error::NotImplemented {
            feature: "macOS domain power (Virtualization.framework path pending)",
        })
    }
}

/// Linux only: whether `virsh` is usable for inventory / power.
#[must_use]
pub fn linux_virsh_available_blocking() -> bool {
    #[cfg(target_os = "linux")]
    {
        crate::kvm::virsh_shell::virsh_version_available_blocking()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use super::*;

    #[test]
    fn list_without_virsh_is_empty_ok() {
        if crate::kvm::virsh_shell::virsh_version_available_blocking() {
            return;
        }
        assert!(list_vms_blocking().unwrap().is_empty());
    }
}
