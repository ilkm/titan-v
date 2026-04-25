//! Apply [`VmSpoofProfile`] steps to an existing VM (Windows / Hyper-V).

#[cfg(windows)]
use std::process::{Command, Stdio};

#[cfg(windows)]
use titan_common::state::VmPowerState;
use titan_common::{Error, Result, VmSpoofProfile};

#[cfg(windows)]
use super::audit::{audit_only, audit_path_from, run_or_record};
#[cfg(windows)]
use super::ps::{
    self, checkpoint_disabled_job_script, dynamic_mac_job_script, enable_vtpm_job_script,
    expose_ve_job_script, processor_count_job_script, secure_boot_template_job_script,
    static_mac_job_script, vlan_access_job_script,
};
#[cfg(windows)]
use super::vm_power::get_vm_power_state_blocking;

#[cfg(windows)]
fn run_ps_capture(vm: &str, script: &str, env: &[(&str, &str)]) -> Result<()> {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script);
    cmd.env("TITAN_VM_NAME", vm);
    for (k, v) in env {
        cmd.env(*k, *v);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("powershell: {e}"),
    })?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::HyperVRejected {
            message: ps::format_ps(out.status.code(), &out.stdout, &out.stderr),
        })
    }
}

#[cfg(windows)]
fn apply_processor_count(
    vm: &str,
    state: VmPowerState,
    n: u32,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if !dry_run && !matches!(state, VmPowerState::Off) {
        tracing::warn!(%vm, "processor_count: VM not Off; apply may fail");
    }
    tracing::info!(%vm, processors = n, "spoof: processor_count");
    if dry_run {
        audit_only(vm, "processor_count", true, audit, steps_out)?;
        return Ok(());
    }
    let pc = n.to_string();
    run_ps_capture(
        vm,
        &processor_count_job_script(),
        &[("TITAN_PROCESSOR_COUNT", pc.as_str())],
    )?;
    audit_only(vm, "processor_count", false, audit, steps_out)
}

#[cfg(windows)]
fn apply_expose_ve(
    vm: &str,
    on: bool,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if dry_run {
        audit_only(
            vm,
            "expose_virtualization_extensions",
            true,
            audit,
            steps_out,
        )?;
        return Ok(());
    }
    run_ps_capture(
        vm,
        &expose_ve_job_script(),
        &[("TITAN_EXPOSE_VE", if on { "true" } else { "false" })],
    )?;
    audit_only(
        vm,
        "expose_virtualization_extensions",
        false,
        audit,
        steps_out,
    )
}

#[cfg(windows)]
fn apply_vlan(
    vm: &str,
    vlan: u16,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if dry_run {
        audit_only(vm, "vlan_access", true, audit, steps_out)?;
        return Ok(());
    }
    let vid = vlan.to_string();
    run_ps_capture(
        vm,
        &vlan_access_job_script(),
        &[("TITAN_VLAN_ID", vid.as_str())],
    )?;
    audit_only(vm, "vlan_access", false, audit, steps_out)
}

#[cfg(windows)]
fn apply_static_mac(
    vm: &str,
    pool_path: &str,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if dry_run {
        audit_only(vm, "static_mac", true, audit, steps_out)?;
        return Ok(());
    }
    run_ps_capture(
        vm,
        &static_mac_job_script(),
        &[("TITAN_MAC_POOL_FILE", pool_path)],
    )?;
    audit_only(vm, "static_mac", false, audit, steps_out)
}

#[cfg(windows)]
fn apply_secure_boot_template(
    vm: &str,
    state: VmPowerState,
    template: &str,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if !matches!(state, VmPowerState::Off) && !dry_run {
        tracing::warn!(%vm, "secure_boot_template: VM should be Off");
    }
    if dry_run {
        audit_only(vm, "secure_boot_template", true, audit, steps_out)?;
        return Ok(());
    }
    run_ps_capture(
        vm,
        &secure_boot_template_job_script(),
        &[("TITAN_SB_TEMPLATE", template.trim())],
    )?;
    audit_only(vm, "secure_boot_template", false, audit, steps_out)
}

#[cfg(windows)]
fn apply_vtpm(
    vm: &str,
    en: bool,
    dry_run: bool,
    audit: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if dry_run {
        audit_only(vm, "vtpm", true, audit, steps_out)?;
        return Ok(());
    }
    run_ps_capture(
        vm,
        &enable_vtpm_job_script(),
        &[("TITAN_VTPM_ENABLE", if en { "true" } else { "false" })],
    )?;
    audit_only(vm, "vtpm", false, audit, steps_out)
}

#[cfg(windows)]
fn apply_profile_checkpoint_processor_ve(
    vm: &str,
    profile: &VmSpoofProfile,
    state: VmPowerState,
    dry_run: bool,
    ap: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if profile.disable_checkpoints {
        run_or_record(
            vm,
            "checkpoint_disabled",
            &checkpoint_disabled_job_script(),
            dry_run,
            ap,
            steps_out,
        )?;
    }
    if let Some(n) = profile.processor_count {
        apply_processor_count(vm, state, n, dry_run, ap, steps_out)?;
    }
    if let Some(on) = profile.expose_virtualization_extensions {
        apply_expose_ve(vm, on, dry_run, ap, steps_out)?;
    }
    Ok(())
}

#[cfg(windows)]
fn apply_profile_vlan_static_mac(
    vm: &str,
    profile: &VmSpoofProfile,
    dry_run: bool,
    ap: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if let Some(vlan) = profile.vlan_id_access {
        apply_vlan(vm, vlan, dry_run, ap, steps_out)?;
    }
    if let Some(ref pool) = profile.static_mac_pool_file {
        apply_static_mac(vm, pool.trim(), dry_run, ap, steps_out)?;
    }
    Ok(())
}

#[cfg(windows)]
fn apply_profile_firmware_mac_tail(
    vm: &str,
    profile: &VmSpoofProfile,
    state: VmPowerState,
    dry_run: bool,
    ap: Option<&std::path::Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    if let Some(ref t) = profile.secure_boot_template {
        apply_secure_boot_template(vm, state, t, dry_run, ap, steps_out)?;
    }
    if let Some(en) = profile.enable_vtpm {
        apply_vtpm(vm, en, dry_run, ap, steps_out)?;
    }
    if profile.dynamic_mac {
        run_or_record(
            vm,
            "dynamic_mac",
            &dynamic_mac_job_script(),
            dry_run,
            ap,
            steps_out,
        )?;
    }
    Ok(())
}

#[cfg(windows)]
fn apply_profile_windows(vm: &str, profile: &VmSpoofProfile, dry_run: bool) -> Result<Vec<String>> {
    let state = get_vm_power_state_blocking(vm)?;
    tracing::info!(%vm, ?state, "spoof: VM power state");
    let audit = audit_path_from(profile);
    let mut steps_out = Vec::new();
    let ap = audit.as_deref();
    apply_profile_checkpoint_processor_ve(vm, profile, state, dry_run, ap, &mut steps_out)?;
    apply_profile_vlan_static_mac(vm, profile, dry_run, ap, &mut steps_out)?;
    apply_profile_firmware_mac_tail(vm, profile, state, dry_run, ap, &mut steps_out)?;
    Ok(steps_out)
}

/// Applies [`VmSpoofProfile`] steps; returns human-readable step markers (including dry-run).
pub fn apply_host_spoof_profile_with_options(
    vm_name: &str,
    profile: &VmSpoofProfile,
    dry_run: bool,
) -> Result<Vec<String>> {
    let vm = vm_name.trim();
    if vm.is_empty() {
        return Err(Error::HyperVRejected {
            message: "vm_name must not be empty".into(),
        });
    }
    profile.validate()?;
    #[cfg(windows)]
    {
        return apply_profile_windows(vm, profile, dry_run);
    }
    #[cfg(not(windows))]
    {
        let _ = vm;
        let _ = profile;
        let _ = dry_run;
        Err(Error::HyperVRejected {
            message: "Hyper-V spoof profile requires Windows with Hyper-V.".into(),
        })
    }
}

/// Back-compat: apply profile with `dry_run = false`.
pub fn apply_host_spoof_profile(vm_name: &str, profile: &VmSpoofProfile) -> Result<()> {
    apply_host_spoof_profile_with_options(vm_name, profile, false).map(|_| ())
}

/// Enables dynamic MAC on all synthetic adapters for an **existing** VM (low risk).
pub fn apply_network_spoof_low_risk(vm_name: &str) -> Result<()> {
    let p = VmSpoofProfile {
        dynamic_mac: true,
        disable_checkpoints: false,
        processor_count: None,
        ..Default::default()
    };
    apply_host_spoof_profile(vm_name, &p)
}
