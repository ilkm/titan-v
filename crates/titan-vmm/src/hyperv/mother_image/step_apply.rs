//! Single-step spoof replay (env-backed optional parameters).

use std::env;

use titan_common::{Error, Result, VmSpoofProfile};

use super::apply::apply_host_spoof_profile_with_options;

fn base_profile() -> VmSpoofProfile {
    VmSpoofProfile {
        dynamic_mac: false,
        disable_checkpoints: false,
        ..VmSpoofProfile::default()
    }
}

fn load_audit_env(p: &mut VmSpoofProfile) {
    if let Ok(audit) = env::var("TITAN_SPOOF_AUDIT_PATH") {
        let t = audit.trim();
        if !t.is_empty() {
            p.audit_log_path = Some(t.into());
        }
    }
}

fn step_processor_count(p: &mut VmSpoofProfile) -> Result<()> {
    let raw = env::var("TITAN_SPOOF_PROCESSOR_COUNT").map_err(|_| Error::HyperVRejected {
        message: "processor_count step: set env TITAN_SPOOF_PROCESSOR_COUNT (u32)".into(),
    })?;
    let n: u32 = raw.trim().parse().map_err(|e| Error::HyperVRejected {
        message: format!("TITAN_SPOOF_PROCESSOR_COUNT: {e}"),
    })?;
    if n == 0 {
        return Err(Error::HyperVRejected {
            message: "TITAN_SPOOF_PROCESSOR_COUNT must be > 0".into(),
        });
    }
    p.processor_count = Some(n);
    Ok(())
}

fn step_expose_ve(p: &mut VmSpoofProfile) {
    let raw = env::var("TITAN_SPOOF_EXPOSE_VE").unwrap_or_else(|_| "true".into());
    let on = matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    );
    p.expose_virtualization_extensions = Some(on);
}

fn step_vlan_access(p: &mut VmSpoofProfile) -> Result<()> {
    let raw = env::var("TITAN_SPOOF_VLAN_ID").map_err(|_| Error::HyperVRejected {
        message: "vlan_access step: set env TITAN_SPOOF_VLAN_ID (u16, 1..=4094)".into(),
    })?;
    let v: u16 = raw.trim().parse().map_err(|e| Error::HyperVRejected {
        message: format!("TITAN_SPOOF_VLAN_ID: {e}"),
    })?;
    p.vlan_id_access = Some(v);
    Ok(())
}

fn step_static_mac(p: &mut VmSpoofProfile) -> Result<()> {
    let path = env::var("TITAN_SPOOF_MAC_POOL_FILE").map_err(|_| Error::HyperVRejected {
        message: "static_mac step: set env TITAN_SPOOF_MAC_POOL_FILE to a text file path".into(),
    })?;
    if path.trim().is_empty() {
        return Err(Error::HyperVRejected {
            message: "TITAN_SPOOF_MAC_POOL_FILE must not be empty".into(),
        });
    }
    p.static_mac_pool_file = Some(path);
    Ok(())
}

fn step_secure_boot_template(p: &mut VmSpoofProfile) -> Result<()> {
    let t = env::var("TITAN_SPOOF_SB_TEMPLATE").map_err(|_| Error::HyperVRejected {
        message: "secure_boot_template step: set env TITAN_SPOOF_SB_TEMPLATE (firmware template)"
            .into(),
    })?;
    if t.trim().is_empty() {
        return Err(Error::HyperVRejected {
            message: "TITAN_SPOOF_SB_TEMPLATE must not be empty".into(),
        });
    }
    p.secure_boot_template = Some(t);
    Ok(())
}

fn step_vtpm(p: &mut VmSpoofProfile) {
    let raw = env::var("TITAN_SPOOF_VTPM_ENABLE").unwrap_or_else(|_| "true".into());
    let on = matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    );
    p.enable_vtpm = Some(on);
}

fn apply_step_id(p: &mut VmSpoofProfile, step_id: &str) -> Result<()> {
    match step_id.trim() {
        "dynamic_mac" => p.dynamic_mac = true,
        "checkpoint_disabled" => p.disable_checkpoints = true,
        "processor_count" => step_processor_count(p)?,
        "expose_ve" | "expose_virtualization_extensions" => step_expose_ve(p),
        "vlan_access" => step_vlan_access(p)?,
        "static_mac" => step_static_mac(p)?,
        "secure_boot_template" => step_secure_boot_template(p)?,
        "vtpm" => step_vtpm(p),
        _ => {
            return Err(Error::HyperVRejected {
                message: format!(
                    "unknown spoof step_id: {step_id} (try dynamic_mac, checkpoint_disabled, or env-backed steps; full profile: ApplySpoofProfile)"
                ),
            });
        }
    }
    Ok(())
}

/// Applies one spoof step by id.
///
/// Steps without extra parameters: `dynamic_mac`, `checkpoint_disabled`.
/// Optional env (operator / debug, single-step replay): `TITAN_SPOOF_PROCESSOR_COUNT`,
/// `TITAN_SPOOF_EXPOSE_VE`, `TITAN_SPOOF_VLAN_ID`, `TITAN_SPOOF_MAC_POOL_FILE`,
/// `TITAN_SPOOF_SB_TEMPLATE`, `TITAN_SPOOF_VTPM_ENABLE`, `TITAN_SPOOF_AUDIT_PATH`.
pub fn apply_spoof_step(vm_name: &str, step_id: &str, dry_run: bool) -> Result<bool> {
    let mut p = base_profile();
    load_audit_env(&mut p);
    apply_step_id(&mut p, step_id)?;
    p.validate()?;
    apply_host_spoof_profile_with_options(vm_name, &p, dry_run)?;
    Ok(true)
}
