//! Host-side VM spoof profile carried on the control plane (`ApplySpoofProfile`).

use std::path::Path;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Host-side VM policy tweaks referenced by [`crate::wire::ControlRequest::ApplySpoofProfile`].
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[serde(default)]
pub struct VmSpoofProfile {
    /// Synthetic NICs: enable dynamic MAC on the guest network adapter.
    pub dynamic_mac: bool,
    /// When true: disable VM checkpoints / snapshots on the host.
    pub disable_checkpoints: bool,
    /// When set: desired vCPU count (VM may need to be off; host enforces best-effort).
    pub processor_count: Option<u32>,
    /// Text file: one static MAC per line (no separators); first adapter uses first line (best-effort).
    pub static_mac_pool_file: Option<String>,
    /// Access VLAN id on synthetic NICs.
    pub vlan_id_access: Option<u16>,
    /// When set: expose nested virtualization extensions to the guest.
    pub expose_virtualization_extensions: Option<bool>,
    /// Secure boot template identifier when VM is off (host-specific string).
    pub secure_boot_template: Option<String>,
    /// Enable guest vTPM / firmware TPM when supported.
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

fn validate_spoof_processor_count(n: Option<u32>) -> Result<()> {
    if let Some(0) = n {
        return Err(Error::InvalidPlan(
            "spoof.processor_count must be > 0 when set".into(),
        ));
    }
    Ok(())
}

fn validate_spoof_vlan_access(v: Option<u16>) -> Result<()> {
    let Some(v) = v else {
        return Ok(());
    };
    if v == 0 || v > 4094 {
        return Err(Error::InvalidPlan(
            "spoof.vlan_id_access must be 1..=4094 when set".into(),
        ));
    }
    Ok(())
}

fn validate_spoof_static_mac_pool_path(p: &Option<String>) -> Result<()> {
    let Some(path) = p else {
        return Ok(());
    };
    let t = path.trim();
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
    Ok(())
}

fn validate_spoof_secure_boot_non_empty(t: &Option<String>) -> Result<()> {
    let Some(t) = t else {
        return Ok(());
    };
    if t.trim().is_empty() {
        return Err(Error::InvalidPlan(
            "spoof.secure_boot_template must not be empty when set".into(),
        ));
    }
    Ok(())
}

impl VmSpoofProfile {
    /// Validates spoof fields that can be checked without a live VMM connection.
    pub fn validate(&self) -> Result<()> {
        validate_spoof_processor_count(self.processor_count)?;
        validate_spoof_vlan_access(self.vlan_id_access)?;
        validate_spoof_static_mac_pool_path(&self.static_mac_pool_file)?;
        validate_spoof_secure_boot_non_empty(&self.secure_boot_template)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_default_ok() {
        VmSpoofProfile::default().validate().unwrap();
    }

    #[test]
    fn validate_rejects_processor_zero() {
        let mut s = VmSpoofProfile::default();
        s.processor_count = Some(0);
        assert!(s.validate().is_err());
    }

    #[test]
    fn validate_rejects_vlan_out_of_range() {
        let mut s = VmSpoofProfile::default();
        s.vlan_id_access = Some(5000);
        assert!(s.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_secure_boot_template() {
        let mut s = VmSpoofProfile::default();
        s.secure_boot_template = Some("   ".into());
        assert!(s.validate().is_err());
    }
}
