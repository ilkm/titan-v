//! **ListVms** and **domain power** for `titan-host`. On Windows this delegates to Hyper-V;
//! off-Windows, stubs exist so workspace crates can type-check on CI / dev machines.

use titan_common::{state::VmPowerState, Error, Result};

/// Lists VMs for the control plane / telemetry.
pub fn list_vms_blocking() -> Result<Vec<(String, VmPowerState)>> {
    #[cfg(windows)]
    {
        return crate::hyperv::list_vms_blocking();
    }
    #[cfg(not(windows))]
    {
        tracing::debug!("platform_vm list: non-Windows build; empty VM list");
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
    #[cfg(not(windows))]
    {
        let _ = (vm_name, start);
        Err(Error::NotImplemented {
            feature: "VM power (Hyper-V only; non-Windows stub build)",
        })
    }
}
