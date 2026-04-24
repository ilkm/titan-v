//! VM provisioning plan validated in user space before Hyper-V calls.

use std::path::Path;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Protocol / schema version for persisted plans and future RPC (see `PROTOCOL_VERSION`).
pub const PLAN_FORMAT_VERSION: u32 = 1;

fn default_auto_start_after_provision() -> bool {
    true
}

fn default_injection_channel() -> String {
    "guest_agent_only".into()
}

/// Declares **intent** for deeper identity / driver paths (need.md 方案 B); defaults are conservative.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct VmIdentityProfile {
    /// Operator expects a host kernel driver IPC channel (see driver bridge probe).
    pub host_kernel_driver_expected: bool,
    /// Desired guest Secure Boot policy when firmware cmdlets apply (`None` = leave unchanged).
    pub guest_secure_boot: Option<bool>,
    /// Enable vTPM when firmware cmdlets apply (`None` = leave unchanged).
    pub guest_vtpm: Option<bool>,
    /// Request offline hive stamping pipeline before first boot (`titan-offline-spoof`).
    pub offline_hive_stamp_requested: bool,
    /// `guest_agent_only` \| `driver_preferred` (future: prefer VMBus driver when present).
    pub injection_channel: String,
}

impl Default for VmIdentityProfile {
    fn default() -> Self {
        Self {
            host_kernel_driver_expected: false,
            guest_secure_boot: None,
            guest_vtpm: None,
            offline_hive_stamp_requested: false,
            injection_channel: default_injection_channel(),
        }
    }
}

impl VmIdentityProfile {
    pub fn validate(&self) -> Result<()> {
        let ch = self.injection_channel.trim();
        if ch != "guest_agent_only" && ch != "driver_preferred" {
            return Err(Error::InvalidPlan(format!(
                "identity.injection_channel must be guest_agent_only or driver_preferred (got {ch})"
            )));
        }
        Ok(())
    }
}

/// Host-side Hyper-V tweaks applied after the VM exists (Layer A; PowerShell).
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[serde(default)]
pub struct VmSpoofProfile {
    /// Synthetic NICs: enable dynamic MAC (`Set-VMNetworkAdapter -DynamicMacAddress On`).
    pub dynamic_mac: bool,
    /// When true: disable checkpoints (`Set-VM -CheckpointType Disabled`).
    pub disable_checkpoints: bool,
    /// When set: `Set-VM -ProcessorCount` (VM may need to be off; host enforces best-effort).
    pub processor_count: Option<u32>,
    /// Text file: one static MAC per line (no separators); first adapter uses first line (best-effort).
    pub static_mac_pool_file: Option<String>,
    /// Access VLAN id on synthetic NICs (`Set-VMNetworkAdapterVlanConfiguration -Access -VlanId`).
    pub vlan_id_access: Option<u16>,
    /// `Set-VMProcessor -ExposeVirtualizationExtensions` when set.
    pub expose_virtualization_extensions: Option<bool>,
    /// `Set-VMFirmware -SecureBootTemplate` value when VM is off (e.g. `MicrosoftWindows`).
    pub secure_boot_template: Option<String>,
    /// Enable guest TPM (`Enable-VMTPM` / firmware policy) when supported.
    pub enable_vtpm: Option<bool>,
    /// Append JSONL audit records for each applied step (host path).
    pub audit_log_path: Option<String>,
    /// Reserved for guest identity pipelines; orchestrator / control plane may trigger artifact work.
    pub guest_identity_tag: Option<String>,
}

impl Default for VmSpoofProfile {
    fn default() -> Self {
        Self {
            dynamic_mac: true,
            disable_checkpoints: false,
            processor_count: None,
            static_mac_pool_file: None,
            vlan_id_access: None,
            expose_virtualization_extensions: None,
            secure_boot_template: None,
            enable_vtpm: None,
            audit_log_path: None,
            guest_identity_tag: None,
        }
    }
}

impl VmSpoofProfile {
    /// Validates spoof fields that can be checked without Hyper-V.
    pub fn validate(&self) -> Result<()> {
        if let Some(n) = self.processor_count {
            if n == 0 {
                return Err(Error::InvalidPlan(
                    "spoof.processor_count must be > 0 when set".into(),
                ));
            }
        }
        if let Some(v) = self.vlan_id_access {
            if v == 0 || v > 4094 {
                return Err(Error::InvalidPlan(
                    "spoof.vlan_id_access must be 1..=4094 when set".into(),
                ));
            }
        }
        if let Some(ref p) = self.static_mac_pool_file {
            let t = p.trim();
            if t.is_empty() {
                return Err(Error::InvalidPlan(
                    "spoof.static_mac_pool_file must not be empty when set".into(),
                ));
            }
            if !Path::new(t).is_file() {
                return Err(Error::InvalidPlan(format!(
                    "spoof.static_mac_pool_file is not a readable file: {t}"
                )));
            }
        }
        if let Some(ref t) = self.secure_boot_template {
            if t.trim().is_empty() {
                return Err(Error::InvalidPlan(
                    "spoof.secure_boot_template must not be empty when set".into(),
                ));
            }
        }
        Ok(())
    }
}

/// A single VM: differencing disk from parent + Gen2 VM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmProvisionPlan {
    /// Absolute or relative path to the read-only parent VHDX.
    pub parent_vhdx: String,
    /// Directory where the differencing `{vm_name}.vhdx` will be created.
    pub diff_dir: String,
    /// Hyper-V VM name (also used as the differencing disk file stem).
    pub vm_name: String,
    pub memory_bytes: u64,
    /// Must be `2` (Generation 2). Reserved for forward compatibility.
    pub generation: u8,
    /// When set, the first synthetic adapter is connected to this vSwitch.
    pub switch_name: Option<String>,
    /// GPU-PV DDA instance path for `Add-VMGpuPartitionAdapter -InstancePath` (omit to skip).
    #[serde(default)]
    pub gpu_partition_instance_path: Option<String>,
    /// After successful `New-VM`, run post steps including `Start-VM` when true (need.md one-click).
    #[serde(default = "default_auto_start_after_provision")]
    pub auto_start_after_provision: bool,
    /// Host-side spoof profile (checkpoint / CPU / NIC policy); see [`VmSpoofProfile`].
    #[serde(default)]
    pub spoof: VmSpoofProfile,
    /// Identity / driver / offline intent (need.md 方案 B traceability).
    #[serde(default)]
    pub identity: VmIdentityProfile,
}

impl VmProvisionPlan {
    /// Validates fields that can be checked without touching the filesystem.
    pub fn validate(&self) -> Result<()> {
        if self.parent_vhdx.trim().is_empty() {
            return Err(Error::InvalidPlan("parent_vhdx must not be empty".into()));
        }
        if self.diff_dir.trim().is_empty() {
            return Err(Error::InvalidPlan("diff_dir must not be empty".into()));
        }
        if self.vm_name.trim().is_empty() {
            return Err(Error::InvalidPlan("vm_name must not be empty".into()));
        }
        if self.memory_bytes == 0 {
            return Err(Error::InvalidPlan("memory_bytes must be > 0".into()));
        }
        if self.generation != 2 {
            return Err(Error::InvalidPlan(format!(
                "only generation 2 is supported (got {})",
                self.generation
            )));
        }
        for label in ["..", "/", "\\"] {
            if self.vm_name.contains(label) {
                return Err(Error::InvalidPlan(format!(
                    "vm_name must not contain {:?}",
                    label
                )));
            }
        }
        if let Some(ref p) = self.gpu_partition_instance_path {
            if p.trim().is_empty() {
                return Err(Error::InvalidPlan(
                    "gpu_partition_instance_path must not be empty when set".into(),
                ));
            }
        }
        self.spoof.validate()?;
        self.identity.validate()?;
        Ok(())
    }

    /// Full path to the differencing VHDX file.
    #[must_use]
    pub fn differencing_vhdx_path(&self) -> String {
        Path::new(self.diff_dir.trim_end_matches(['/', '\\']))
            .join(format!("{}.vhdx", self.vm_name.trim()))
            .to_string_lossy()
            .into_owned()
    }

    /// Returns `true` if `parent_vhdx` exists on disk.
    #[must_use]
    pub fn parent_exists(&self) -> bool {
        Path::new(self.parent_vhdx.trim()).is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan() -> VmProvisionPlan {
        VmProvisionPlan {
            parent_vhdx: "D:\\Images\\parent.vhdx".into(),
            diff_dir: "D:\\Diffs".into(),
            vm_name: "vm-01".into(),
            memory_bytes: 4 * 1024 * 1024 * 1024,
            generation: 2,
            switch_name: Some("External".into()),
            gpu_partition_instance_path: None,
            auto_start_after_provision: true,
            spoof: VmSpoofProfile::default(),
            identity: VmIdentityProfile::default(),
        }
    }

    #[test]
    fn validate_ok() {
        sample_plan().validate().unwrap();
    }

    #[test]
    fn validate_rejects_gen1() {
        let mut p = sample_plan();
        p.generation = 1;
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_gpu_path() {
        let mut p = sample_plan();
        p.gpu_partition_instance_path = Some("  ".into());
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_rejects_spoof_processor_zero() {
        let mut p = sample_plan();
        p.spoof.processor_count = Some(0);
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_rejects_bad_injection_channel() {
        let mut p = sample_plan();
        p.identity.injection_channel = "nope".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn differencing_path_joins() {
        let p = VmProvisionPlan {
            parent_vhdx: "p.vhdx".into(),
            diff_dir: "C:\\tmp/".into(),
            vm_name: "n".into(),
            memory_bytes: 1,
            generation: 2,
            switch_name: None,
            gpu_partition_instance_path: None,
            auto_start_after_provision: true,
            spoof: VmSpoofProfile::default(),
            identity: VmIdentityProfile::default(),
        };
        let joined = p.differencing_vhdx_path();
        assert!(joined.ends_with("n.vhdx"), "joined={joined}");
        assert!(joined.contains("tmp"), "joined={joined}");
    }
}
