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
    let rows = spawn_blocking_result(titan_vmm::platform_vm::list_vms_blocking).await?;
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

#[cfg(windows)]
fn spoof_apply_notes(dry_run: bool) -> String {
    if dry_run {
        "dry-run: steps were recorded; Hyper-V mutations skipped where applicable".into()
    } else {
        String::new()
    }
}

#[cfg(windows)]
async fn handle_apply_spoof_profile_windows(
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

async fn handle_apply_spoof_profile(
    vm_name: String,
    dry_run: bool,
    spoof: VmSpoofProfile,
) -> Result<ControlResponse, ServeError> {
    #[cfg(not(windows))]
    {
        let _ = dry_run;
        let _ = spoof;
        Ok(server_err(
            501,
            format!(
                "ApplySpoofProfile requires Windows with Hyper-V (mother_image); vm_name={vm_name}"
            ),
        ))
    }
    #[cfg(windows)]
    {
        handle_apply_spoof_profile_windows(vm_name, dry_run, spoof).await
    }
}

#[cfg(windows)]
async fn handle_apply_spoof_step_windows(
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

async fn handle_apply_spoof_step(
    vm_name: String,
    step_id: String,
    dry_run: bool,
) -> Result<ControlResponse, ServeError> {
    #[cfg(not(windows))]
    {
        let _ = dry_run;
        Ok(ControlResponse::SpoofStepAck {
            vm_name,
            step_id,
            dry_run,
            ok: false,
            detail: "ApplySpoofStep requires Windows with Hyper-V (mother_image).".into(),
        })
    }
    #[cfg(windows)]
    {
        handle_apply_spoof_step_windows(vm_name, step_id, dry_run).await
    }
}

async fn dispatch_vm_requests(
    req: ControlRequest,
    request_id: &str,
    state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    match req {
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
        _ => Err(ServeError::Io(std::io::Error::other(
            "internal: dispatch_vm_requests",
        ))),
    }
}

async fn dispatch_spoof_host_requests(req: ControlRequest) -> Result<ControlResponse, ServeError> {
    match req {
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
        ControlRequest::HostDesktopSnapshot {
            max_width,
            max_height,
            jpeg_quality,
        } => handle_host_desktop_snapshot(max_width, max_height, jpeg_quality).await,
        ControlRequest::HostResourceSnapshot => handle_host_resource_snapshot().await,
        _ => Err(ServeError::Io(std::io::Error::other(
            "internal: dispatch_spoof_host_requests",
        ))),
    }
}

async fn dispatch_request_rest(
    req: ControlRequest,
    request_id: &str,
    state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    match req {
        ControlRequest::ApplyHostUiPersistJson { json } => {
            super::apply_host_ui::handle_apply_host_ui_persist_json(json, state).await
        }
        ControlRequest::SetUiLang { lang } => super::apply_host_ui::handle_set_ui_lang(lang, state),
        ControlRequest::Ping | ControlRequest::Hello => Err(ServeError::Io(std::io::Error::other(
            "internal: dispatch_request_rest received Ping/Hello",
        ))),
        ControlRequest::ListVms
        | ControlRequest::StartVmGroup { .. }
        | ControlRequest::StopVmGroup { .. }
        | ControlRequest::SetScriptArtifact { .. }
        | ControlRequest::LoadScriptVm { .. } => dispatch_vm_requests(req, request_id, state).await,
        ControlRequest::ApplySpoofProfile { .. }
        | ControlRequest::ApplySpoofStep { .. }
        | ControlRequest::HostDesktopSnapshot { .. }
        | ControlRequest::HostResourceSnapshot => dispatch_spoof_host_requests(req).await,
    }
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
        other => dispatch_request_rest(other, request_id, state).await,
    }
}

async fn handle_host_resource_snapshot() -> Result<ControlResponse, ServeError> {
    let join = tokio::task::spawn_blocking(crate::host_resources::collect_blocking)
        .await
        .map_err(|e| join_io(format!("join: {e}")))?;
    Ok(ControlResponse::HostResourceSnapshot { stats: join })
}

fn desktop_jpeg_response_or_limit(
    jpeg_bytes: Vec<u8>,
    width_px: u32,
    height_px: u32,
) -> Result<ControlResponse, ServeError> {
    let max = titan_common::MAX_PAYLOAD_BYTES as usize;
    if jpeg_bytes.len() > max.saturating_sub(512) {
        return Ok(server_err(
            413,
            format!(
                "desktop JPEG {} bytes exceeds wire limit (~{} bytes); lower resolution or quality",
                jpeg_bytes.len(),
                max
            ),
        ));
    }
    Ok(ControlResponse::DesktopSnapshotJpeg {
        jpeg_bytes,
        width_px,
        height_px,
    })
}

async fn handle_host_desktop_snapshot(
    max_width: u32,
    max_height: u32,
    jpeg_quality: u8,
) -> Result<ControlResponse, ServeError> {
    let mw = max_width.clamp(320, 4096);
    let mh = max_height.clamp(240, 4096);
    let q = jpeg_quality.clamp(1, 95);
    let join = tokio::task::spawn_blocking(move || {
        crate::desktop_snapshot::capture_primary_display_jpeg(mw, mh, q)
    })
    .await
    .map_err(|e| join_io(format!("join: {e}")))?;
    match join {
        Ok((jpeg_bytes, width_px, height_px)) => {
            desktop_jpeg_response_or_limit(jpeg_bytes, width_px, height_px)
        }
        Err(e) => Ok(server_err(500, e)),
    }
}
