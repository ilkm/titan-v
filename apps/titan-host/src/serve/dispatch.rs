use titan_common::{ControlRequest, ControlResponse, VmSpoofProfile};

use super::errors::ServeError;
use super::response::server_err;
use super::state::ServeState;

fn join_io(err: impl std::fmt::Display) -> ServeError {
    ServeError::Io(std::io::Error::other(err.to_string()))
}

const BATCH_POWER_REMOVED: &str =
    "StartVmGroup/StopVmGroup: OpenVMM / VM batch power not wired in this build.";
const SCRIPT_REMOVED: &str =
    "Lua script execution path removed from this build (SetScriptArtifact / LoadScriptVm).";
const SPOOF_REMOVED: &str =
    "ApplySpoofProfile/ApplySpoofStep: mother_image / host spoof pipeline not wired in this build.";

async fn handle_list_vms() -> Result<ControlResponse, ServeError> {
    Ok(ControlResponse::VmList { vms: Vec::new() })
}

async fn handle_start_vm_group(_vm_names: Vec<String>) -> Result<ControlResponse, ServeError> {
    Ok(ControlResponse::BatchPowerAck {
        succeeded: 0,
        failures: vec![BATCH_POWER_REMOVED.into()],
    })
}

async fn handle_stop_vm_group(_vm_names: Vec<String>) -> Result<ControlResponse, ServeError> {
    Ok(ControlResponse::BatchPowerAck {
        succeeded: 0,
        failures: vec![BATCH_POWER_REMOVED.into()],
    })
}

fn handle_set_script_artifact() -> Result<ControlResponse, ServeError> {
    Ok(server_err(501, SCRIPT_REMOVED))
}

fn handle_load_script_vm() -> Result<ControlResponse, ServeError> {
    Ok(server_err(501, SCRIPT_REMOVED))
}

fn handle_apply_spoof_profile(
    _vm_name: String,
    _dry_run: bool,
    spoof: VmSpoofProfile,
) -> Result<ControlResponse, ServeError> {
    if let Err(e) = spoof.validate() {
        return Ok(server_err(400, format!("invalid VmSpoofProfile: {e}")));
    }
    Ok(server_err(501, SPOOF_REMOVED))
}

fn handle_apply_spoof_step(
    vm_name: String,
    step_id: String,
    dry_run: bool,
) -> Result<ControlResponse, ServeError> {
    Ok(ControlResponse::SpoofStepAck {
        vm_name,
        step_id,
        dry_run,
        ok: false,
        detail: SPOOF_REMOVED.into(),
    })
}

async fn dispatch_vm_requests(
    req: ControlRequest,
    _request_id: &str,
    _state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    match req {
        ControlRequest::ListVms => handle_list_vms().await,
        ControlRequest::StartVmGroup { vm_names } => handle_start_vm_group(vm_names).await,
        ControlRequest::StopVmGroup { vm_names } => handle_stop_vm_group(vm_names).await,
        ControlRequest::SetScriptArtifact { .. } => handle_set_script_artifact(),
        ControlRequest::LoadScriptVm { .. } => handle_load_script_vm(),
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
        } => handle_apply_spoof_profile(vm_name, dry_run, spoof),
        ControlRequest::ApplySpoofStep {
            vm_name,
            step_id,
            dry_run,
        } => handle_apply_spoof_step(vm_name, step_id, dry_run),
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

fn is_vm_request(req: &ControlRequest) -> bool {
    matches!(
        req,
        ControlRequest::ListVms
            | ControlRequest::StartVmGroup { .. }
            | ControlRequest::StopVmGroup { .. }
            | ControlRequest::SetScriptArtifact { .. }
            | ControlRequest::LoadScriptVm { .. }
    )
}

fn is_spoof_host_request(req: &ControlRequest) -> bool {
    matches!(
        req,
        ControlRequest::ApplySpoofProfile { .. }
            | ControlRequest::ApplySpoofStep { .. }
            | ControlRequest::HostDesktopSnapshot { .. }
            | ControlRequest::HostResourceSnapshot
    )
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
        ControlRequest::ApplyVmWindowSnapshot {
            device_id,
            records_json,
        } => {
            super::vm_window_remote::handle_apply_vm_window_snapshot(device_id, records_json, state)
                .await
        }
        ControlRequest::SubscribeTelemetry => {
            Ok(ControlResponse::SubscribeTelemetryAck { ok: true })
        }
        ControlRequest::Ping | ControlRequest::Hello => Err(ServeError::Io(std::io::Error::other(
            "internal: dispatch_request_rest received Ping/Hello",
        ))),
        other if is_vm_request(&other) => dispatch_vm_requests(other, request_id, state).await,
        other if is_spoof_host_request(&other) => dispatch_spoof_host_requests(other).await,
        other => Err(ServeError::Io(std::io::Error::other(format!(
            "internal: unhandled ControlRequest variant {other:?}"
        )))),
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
