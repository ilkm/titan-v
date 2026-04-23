//! Host node CLI: Hyper-V M1 provisioning and orchestration stubs.

mod config;
mod orchestrator;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};
use titan_common::{VmProvisionPlan, VmSpoofProfile};
use titan_host::serve;
use tracing_subscriber::EnvFilter;

use crate::config::HostConfigFile;
use crate::orchestrator::Orchestrator;

#[derive(Parser)]
#[command(
    name = "titan-host",
    version,
    about = "Titan-v host node (Hyper-V M1 provisioning)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create differencing VHDX + Generation 2 VM from a TOML config.
    Provision {
        /// Path to host.toml (see example in repository tests).
        #[arg(long, value_name = "FILE")]
        config: PathBuf,
        /// Stop on first VM provisioning failure (default: continue and report counts).
        #[arg(long, action = clap::ArgAction::SetTrue)]
        fail_fast: bool,
        /// Validate plans and parent VHDX paths only; do not call Hyper-V.
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    /// Listen for center control connections (M2 wire protocol).
    Serve {
        /// Listen address (e.g. 127.0.0.1:7788).
        #[arg(long, default_value = "127.0.0.1:7788")]
        listen: SocketAddr,
        /// Optional `agent-bindings.toml` (VM name → guest agent TCP address).
        #[arg(long, value_name = "FILE")]
        agent_bindings: Option<PathBuf>,
    },
    /// Run Lua tooling (mlua).
    Script {
        #[command(subcommand)]
        cmd: ScriptCmd,
    },
    /// Mother-image de-fingerprint planning surface (Phase 3; operations are NYI).
    Spoof {
        #[command(subcommand)]
        cmd: SpoofCmd,
    },
}

#[derive(Subcommand)]
enum SpoofCmd {
    /// Write a JSON mother-image checklist for a template VM name.
    Plan {
        #[arg(long)]
        vm_template: String,
        /// Output directory for the plan JSON (default: system temp).
        #[arg(long, value_name = "DIR")]
        output: Option<PathBuf>,
    },
    /// Apply a host-side [`VmSpoofProfile`] to an existing VM (PowerShell; Windows only).
    Apply {
        #[arg(long)]
        vm_name: String,
        #[arg(long, default_value_t = true)]
        dynamic_mac: bool,
        #[arg(long, default_value_t = false)]
        disable_checkpoints: bool,
        #[arg(long)]
        processor_count: Option<u32>,
    },
}

#[derive(Subcommand)]
enum ScriptCmd {
    /// Execute a `.lua` file once (no network; size-capped).
    Eval {
        /// Path to the script source.
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Provision {
            config,
            fail_fast,
            dry_run,
        } => run_provision(&config, fail_fast, dry_run).await?,
        Commands::Serve {
            listen,
            agent_bindings,
        } => {
            serve::run_serve(listen, agent_bindings)
                .await
                .map_err(|e| anyhow::anyhow!("serve: {e}"))?;
        }
        Commands::Script {
            cmd: ScriptCmd::Eval { file },
        } => run_script_eval(&file)?,
        Commands::Spoof { cmd } => run_spoof(cmd)?,
    }
    Ok(())
}

fn run_spoof(cmd: SpoofCmd) -> anyhow::Result<()> {
    match cmd {
        SpoofCmd::Plan {
            vm_template,
            output,
        } => {
            let dir = output.unwrap_or_else(std::env::temp_dir);
            let path = titan_vmm::hyperv::mother_image::plan_mother_image_spoof(&vm_template, &dir)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            tracing::warn!(
                path = %path.display(),
                vm_template = %vm_template,
                "mother-image plan written; review game EULA and law before automated guest tweaks"
            );
        }
        SpoofCmd::Apply {
            vm_name,
            dynamic_mac,
            disable_checkpoints,
            processor_count,
        } => {
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
        }
    }
    Ok(())
}

const MAX_SCRIPT_BYTES: u64 = 256 * 1024;

fn run_script_eval(path: &Path) -> anyhow::Result<()> {
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

async fn run_provision(
    config_path: &PathBuf,
    fail_fast: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = HostConfigFile::load(config_path)
        .with_context(|| format!("load host config {}", config_path.display()))?;
    let plans = cfg
        .expanded_vm_plans()
        .with_context(|| format!("expand vm/vm_group in {}", config_path.display()))?;
    if plans.is_empty() {
        anyhow::bail!("config has no VMs: add [[vm]] and/or [[vm_group]] entries");
    }

    if dry_run {
        for p in &plans {
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
        return Ok(());
    }

    Orchestrator::log_post_m1_pipeline();

    let per_vm_timeout = cfg.timeout();
    let mut ok = 0u32;
    let mut fail = 0u32;
    for plan in &plans {
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
            let hyperv = titan_vmm::hyperv::HypervBackend;
            hyperv.provision_plan_blocking(&plan, per_vm_timeout)?;
            Orchestrator::post_provision_after_create(&plan, fail_fast)
        }),
    )
    .await
    {
        Ok(join) => {
            let inner = join.map_err(|e| titan_common::Error::HyperVRejected {
                message: format!("provision task join error for {label}: {e}"),
            })?;
            inner
        }
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
