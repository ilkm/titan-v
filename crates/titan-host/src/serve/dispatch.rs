use titan_common::{ControlRequest, ControlResponse, VmBrief, VmSpoofProfile};

use super::errors::ServeError;
use super::limits::MAX_SCRIPT_SOURCE_BYTES;
use super::power::batch_power;
use super::response::server_err;
use super::state::{script_artifact_cell, try_enqueue_script_vm, ServeState};

fn join_io(err: impl std::fmt::Display) -> ServeError {
    ServeError::Io(std::io::Error::other(err.to_string()))
}

async fn spawn_blocking_result<T, E, F>(f: F) -> Result<T, ServeError>
where
    F: FnOnce() -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| join_io(format!("join: {e}")))?
        .map_err(|e| ServeError::Io(std::io::Error::other(e.to_string())))
}

async fn spawn_blocking_value<T, F>(f: F) -> Result<T, ServeError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| join_io(format!("join: {e}")))
}

async fn handle_list_vms() -> Result<ControlResponse, ServeError> {
    let rows = spawn_blocking_result(titan_vmm::hyperv::list_vms_blocking).await?;
    let vms = rows
        .into_iter()
        .map(|(name, s)| VmBrief { name, state: s })
        .collect();
    Ok(ControlResponse::VmList { vms })
}

async fn handle_start_vm_group(vm_names: Vec<String>) -> Result<ControlResponse, ServeError> {
    let report = spawn_blocking_value(move || batch_power(true, &vm_names)).await?;
    Ok(ControlResponse::BatchPowerAck {
        succeeded: report.succeeded,
        failures: report.failures,
    })
}

async fn handle_stop_vm_group(vm_names: Vec<String>) -> Result<ControlResponse, ServeError> {
    let report = spawn_blocking_value(move || batch_power(false, &vm_names)).await?;
    Ok(ControlResponse::BatchPowerAck {
        succeeded: report.succeeded,
        failures: report.failures,
    })
}

fn handle_set_script_artifact(
    request_id: &str,
    version: String,
    sha256_hex: String,
) -> Result<ControlResponse, ServeError> {
    let v = version.chars().take(256).collect::<String>();
    let h = sha256_hex.chars().take(128).collect::<String>();
    let mut g = script_artifact_cell()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *g = Some((v.clone(), h));
    tracing::info!(%request_id, script_version = %v, "script artifact recorded");
    Ok(ControlResponse::ScriptArtifactAck { version: v })
}

fn handle_load_script_vm(
    request_id: &str,
    state: &ServeState,
    vm_name: String,
    source: String,
) -> Result<ControlResponse, ServeError> {
    if source.len() > MAX_SCRIPT_SOURCE_BYTES {
        return Ok(server_err(
            413,
            format!("script source exceeds {MAX_SCRIPT_SOURCE_BYTES} bytes"),
        ));
    }
    let res = try_enqueue_script_vm(&state.script_tx, vm_name, source);
    if let ControlResponse::ScriptLoadAck { vm_name } = &res {
        tracing::info!(%request_id, %vm_name, "script job queued");
    }
    Ok(res)
}

fn spoof_apply_notes(dry_run: bool) -> String {
    if dry_run {
        "dry-run: steps were recorded; Hyper-V mutations skipped where applicable".into()
    } else {
        String::new()
    }
}

async fn handle_apply_spoof_profile(
    vm_name: String,
    dry_run: bool,
    spoof: VmSpoofProfile,
) -> Result<ControlResponse, ServeError> {
    if let Err(e) = spoof.validate() {
        return Ok(server_err(400, format!("invalid VmSpoofProfile: {e}")));
    }
    let vm = vm_name.clone();
    let profile = spoof;
    let join = tokio::task::spawn_blocking(move || {
        titan_vmm::hyperv::mother_image::apply_host_spoof_profile_with_options(
            &vm, &profile, dry_run,
        )
    })
    .await
    .map_err(|e| join_io(format!("join: {e}")))?;
    match join {
        Ok(steps) => Ok(ControlResponse::SpoofApplyAck {
            vm_name,
            dry_run,
            steps_executed: steps,
            notes: spoof_apply_notes(dry_run),
        }),
        Err(e) => Ok(server_err(500, e.to_string())),
    }
}

async fn handle_apply_spoof_step(
    vm_name: String,
    step_id: String,
    dry_run: bool,
) -> Result<ControlResponse, ServeError> {
    let vm = vm_name.clone();
    let sid = step_id.clone();
    let join = tokio::task::spawn_blocking(move || {
        titan_vmm::hyperv::mother_image::apply_spoof_step(&vm, &sid, dry_run)
    })
    .await
    .map_err(|e| join_io(format!("join: {e}")))?;
    match join {
        Ok(ok) => Ok(ControlResponse::SpoofStepAck {
            vm_name,
            step_id,
            dry_run,
            ok,
            detail: "ok".into(),
        }),
        Err(e) => Ok(ControlResponse::SpoofStepAck {
            vm_name,
            step_id,
            dry_run,
            ok: false,
            detail: e.to_string(),
        }),
    }
}

fn handle_register_guest_agent(
    request_id: &str,
    state: &ServeState,
    vm_name: String,
    guest_agent_addr: String,
) -> Result<ControlResponse, ServeError> {
    let vm = vm_name.trim().to_string();
    if vm.is_empty() {
        return Ok(server_err(400, "vm_name is empty"));
    }
    let addr_trim = guest_agent_addr.trim();
    let addr: std::net::SocketAddr = match addr_trim.parse() {
        Ok(a) => a,
        Err(e) => return Ok(server_err(400, format!("invalid guest_agent_addr: {e}"))),
    };
    state.agents.insert(vm.clone(), addr);
    tracing::info!(%request_id, %vm, %addr, "guest agent binding registered");
    if let Some(path) = &state.agent_bindings_path {
        if let Err(e) = crate::agent_bindings::save_agent_bindings(path, &state.agents) {
            tracing::warn!(error = %e, path = %path.display(), "failed to persist agent bindings");
        }
    }
    Ok(ControlResponse::GuestAgentRegisterAck { vm_name: vm })
}

pub(super) async fn dispatch_request(
    req: ControlRequest,
    request_id: &str,
    state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    let caps = state.capabilities();
    match req {
        ControlRequest::Ping => Ok(ControlResponse::Pong { capabilities: caps }),
        ControlRequest::Hello => Ok(ControlResponse::HelloAck { capabilities: caps }),
        ControlRequest::ListVms => handle_list_vms().await,
        ControlRequest::StartVmGroup { vm_names } => handle_start_vm_group(vm_names).await,
        ControlRequest::StopVmGroup { vm_names } => handle_stop_vm_group(vm_names).await,
        ControlRequest::SetScriptArtifact {
            version,
            sha256_hex,
        } => handle_set_script_artifact(request_id, version, sha256_hex),
        ControlRequest::LoadScriptVm { vm_name, source } => {
            handle_load_script_vm(request_id, state, vm_name, source)
        }
        ControlRequest::ApplySpoofProfile {
            vm_name,
            dry_run,
            spoof,
        } => handle_apply_spoof_profile(vm_name, dry_run, spoof).await,
        ControlRequest::ApplySpoofStep {
            vm_name,
            step_id,
            dry_run,
        } => handle_apply_spoof_step(vm_name, step_id, dry_run).await,
        ControlRequest::RegisterGuestAgent {
            vm_name,
            guest_agent_addr,
        } => handle_register_guest_agent(request_id, state, vm_name, guest_agent_addr),
        _ => Ok(server_err(
            501,
            "unsupported control request for this host build",
        )),
    }
}
