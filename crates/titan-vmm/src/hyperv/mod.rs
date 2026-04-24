//! Microsoft Hyper-V backend (Windows): Gen2 + differencing VHDX provisioning (M1).

pub mod gpu_pv;
pub mod guest_agent;
pub mod host_runtime;
pub mod mother_image;

use std::time::Duration;

pub use host_runtime::{
    AgentBindingTable, HypervGpuPartitioner, HypervHardwareSpoofer, HypervHostRuntime,
    HypervStreamPrecheck,
};

use titan_common::{state::VmPowerState, Error, Result, VmProvisionPlan};

use crate::traits::PowerControl;

#[cfg(windows)]
mod windows;

/// Minimal Hyper-V handle: **power + provisioning** only. Use [`HypervHostRuntime`] for guest
/// agent read/inject and [`VmbusInput`](titan_common::VmbusInput) via the same runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct HypervBackend;

impl PowerControl for HypervBackend {
    fn start(&self, vm_id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            windows::vm_power(vm_id, true)
        }
        #[cfg(not(windows))]
        {
            let _ = vm_id;
            Err(Error::HyperVRejected {
                message: "Hyper-V VM start requires Windows with Hyper-V.".into(),
            })
        }
    }

    fn stop(&self, vm_id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            windows::vm_power(vm_id, false)
        }
        #[cfg(not(windows))]
        {
            let _ = vm_id;
            Err(Error::HyperVRejected {
                message: "Hyper-V VM stop requires Windows with Hyper-V.".into(),
            })
        }
    }
}

/// Returns whether a VM exists (`Get-VM`); non-Windows always returns `false`.
pub fn vm_exists_blocking(vm_name: &str) -> Result<bool> {
    #[cfg(windows)]
    {
        windows::vm_exists(vm_name)
    }
    #[cfg(not(windows))]
    {
        let _ = vm_name;
        Ok(false)
    }
}

/// Lists VMs visible to Hyper-V PowerShell (blocking). Non-Windows builds return an empty list.
pub fn list_vms_blocking() -> Result<Vec<(String, VmPowerState)>> {
    #[cfg(windows)]
    {
        windows::list_vms()
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

impl HypervBackend {
    /// Runs provisioning for one plan (blocking); call from `spawn_blocking`.
    pub fn provision_plan_blocking(&self, plan: &VmProvisionPlan, timeout: Duration) -> Result<()> {
        provision_plan_blocking(plan, timeout)
    }
}

/// Runs provisioning for one plan (blocking); call from `spawn_blocking`.
pub fn provision_plan_blocking(plan: &VmProvisionPlan, timeout: Duration) -> Result<()> {
    let _ = timeout;
    plan.validate()?;
    #[cfg(windows)]
    {
        if !super::gpu_pv::hyperv_ps_module_available_blocking() {
            tracing::warn!(
                vm = %plan.vm_name,
                "provision skipped: Hyper-V PowerShell module not available (role not installed or disabled)"
            );
            return Ok(());
        }
        windows::provision(plan)
    }
    #[cfg(not(windows))]
    {
        tracing::warn!(
            vm = %plan.vm_name,
            "provision skipped: Hyper-V automation is not available on this host OS (non-Windows build)"
        );
        Ok(())
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;
    use crate::traits::PowerControl;

    #[test]
    fn power_control_non_windows_rejects() {
        let err = HypervBackend.start("vm-1").unwrap_err();
        assert!(err.to_string().contains("Windows"), "{}", err);
    }

    #[test]
    fn list_vms_empty_off_windows() {
        assert!(list_vms_blocking().unwrap().is_empty());
    }

    #[test]
    fn non_windows_provision_is_noop_ok() {
        let plan = VmProvisionPlan {
            parent_vhdx: "C:\\p.vhdx".into(),
            diff_dir: "C:\\d".into(),
            vm_name: "t".into(),
            memory_bytes: 512 * 1024 * 1024,
            generation: 2,
            switch_name: None,
            gpu_partition_instance_path: None,
            auto_start_after_provision: true,
            spoof: titan_common::VmSpoofProfile::default(),
            identity: titan_common::VmIdentityProfile::default(),
        };
        provision_plan_blocking(&plan, Duration::from_secs(1)).unwrap();
    }
}
