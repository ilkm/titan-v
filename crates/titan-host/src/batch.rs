//! Batch Hyper-V provisioning, Lua eval, and mother-image spoof helpers (library API; no binary CLI).

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use titan_common::VmProvisionPlan;

use crate::config::HostConfigFile;
use crate::orchestrator::Orchestrator;

/// Mother-image checklist / host spoof operations (formerly CLI `spoof` subcommands).
#[derive(Debug)]
pub enum SpoofCommand {
    Plan {
        vm_template: String,
        output: Option<PathBuf>,
    },
    Apply {
        vm_name: String,
        dynamic_mac: bool,
        disable_checkpoints: bool,
        processor_count: Option<u32>,
    },
}

fn run_spoof_plan(vm_template: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    let dir = output.unwrap_or_else(std::env::temp_dir);
    let path = titan_vmm::hyperv::mother_image::plan_mother_image_spoof(&vm_template, &dir)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tracing::warn!(
        path = %path.display(),
        vm_template = %vm_template,
        "mother-image plan written; review game EULA and law before automated guest tweaks"
    );
    Ok(())
}

fn run_spoof_apply(
    vm_name: String,
    dynamic_mac: bool,
    disable_checkpoints: bool,
    processor_count: Option<u32>,
) -> anyhow::Result<()> {
    use titan_common::VmSpoofProfile;

    let profile = VmSpoofProfile {
        dynamic_mac,
        disable_checkpoints,
        processor_count,
        guest_identity_tag: None,
        ..Default::default()
    };
    profile
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid spoof profile: {e}"))?;
    titan_vmm::hyperv::mother_image::apply_host_spoof_profile(&vm_name, &profile)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    tracing::info!(%vm_name, "spoof apply finished");
    Ok(())
}

pub fn run_spoof(cmd: SpoofCommand) -> anyhow::Result<()> {
    match cmd {
        SpoofCommand::Plan {
            vm_template,
            output,
        } => run_spoof_plan(vm_template, output),
        SpoofCommand::Apply {
            vm_name,
            dynamic_mac,
            disable_checkpoints,
            processor_count,
        } => run_spoof_apply(vm_name, dynamic_mac, disable_checkpoints, processor_count),
    }
}

pub const MAX_SCRIPT_BYTES: u64 = 256 * 1024;

pub fn script_eval(path: &Path) -> anyhow::Result<()> {
    let meta = std::fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if meta.len() > MAX_SCRIPT_BYTES {
        anyhow::bail!("script file too large (max {MAX_SCRIPT_BYTES} bytes)");
    }
    let source =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let engine = titan_scripts::ScriptEngine::new().map_err(|e| anyhow::anyhow!("{e}"))?;
    engine
        .exec_chunk(&source)
        .map_err(|e| anyhow::anyhow!("lua execution failed: {e}"))
}

pub async fn run_provision(
    config_path: &PathBuf,
    fail_fast: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = HostConfigFile::load(config_path)
        .with_context(|| format!("load host config {}", config_path.display()))?;
    let plans = cfg
        .expanded_vm_plans()
        .with_context(|| format!("expand vm/vm_group in {}", config_path.display()))?;
    let per_vm_timeout = cfg.timeout();
    run_provision_plans(plans, per_vm_timeout, fail_fast, dry_run).await
}

fn provision_dry_run_validate(plans: &[VmProvisionPlan]) -> anyhow::Result<()> {
    for p in plans {
        p.validate()
            .map_err(|e| anyhow::anyhow!("plan {} invalid: {e}", p.vm_name))?;
        if !p.parent_exists() {
            anyhow::bail!(
                "dry-run: parent VHDX missing for {}: {}",
                p.vm_name,
                p.parent_vhdx
            );
        }
    }
    Orchestrator::log_post_m1_pipeline();
    tracing::info!(count = plans.len(), "provision dry-run ok");
    Ok(())
}

async fn provision_execute_all(
    plans: &[VmProvisionPlan],
    per_vm_timeout: Duration,
    fail_fast: bool,
) -> anyhow::Result<()> {
    Orchestrator::log_post_m1_pipeline();
    let mut ok = 0u32;
    let mut fail = 0u32;
    for plan in plans {
        match run_one_vm(plan.clone(), per_vm_timeout, fail_fast).await {
            Ok(()) => {
                ok += 1;
                tracing::info!(vm = %plan.vm_name, "provision ok");
            }
            Err(e) => {
                fail += 1;
                tracing::error!(vm = %plan.vm_name, error = %e, "provision failed");
                if fail_fast {
                    anyhow::bail!("fail-fast: {}", e);
                }
            }
        }
    }
    tracing::info!(ok, fail, "provision batch finished");
    Ok(())
}

/// Batch provision from an already-expanded plan list (GUI / in-memory; no TOML path).
pub async fn run_provision_plans(
    plans: Vec<VmProvisionPlan>,
    per_vm_timeout: Duration,
    fail_fast: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    if plans.is_empty() {
        anyhow::bail!("no VMs to provision: add explicit VMs and/or vm_group templates");
    }
    if dry_run {
        provision_dry_run_validate(&plans)?;
        return Ok(());
    }
    provision_execute_all(&plans, per_vm_timeout, fail_fast).await
}

fn run_provision_blocking(
    plan: &VmProvisionPlan,
    per_vm_timeout: Duration,
    fail_fast: bool,
) -> titan_common::Result<()> {
    let hyperv = titan_vmm::hyperv::HypervBackend;
    hyperv.provision_plan_blocking(plan, per_vm_timeout)?;
    Orchestrator::post_provision_after_create(plan, fail_fast)
}

async fn run_one_vm(
    plan: VmProvisionPlan,
    per_vm_timeout: Duration,
    fail_fast: bool,
) -> titan_common::Result<()> {
    plan.validate()?;
    let label = plan.vm_name.clone();
    match tokio::time::timeout(
        per_vm_timeout,
        tokio::task::spawn_blocking(move || {
            run_provision_blocking(&plan, per_vm_timeout, fail_fast)
        }),
    )
    .await
    {
        Ok(join) => join.map_err(|e| titan_common::Error::HyperVRejected {
            message: format!("provision task join error for {label}: {e}"),
        })?,
        Err(_elapsed) => {
            tracing::warn!(
                target: "titan_host::hyperv",
                vm = %label,
                "outer timeout elapsed; powershell.exe may still be running"
            );
            Err(titan_common::Error::Timeout(per_vm_timeout))
        }
    }
}
