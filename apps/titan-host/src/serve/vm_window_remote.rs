//! Host-side handler for [`titan_common::ControlRequest::ApplyVmWindowSnapshot`].
//!
//! Center is the sole source of truth (Center-side SQLite); the host only renders rows pushed by
//! Center. Each call replaces the in-memory list on the egui thread (via
//! [`VmWindowReloadMsg::Replace`]) and acks with the row count actually adopted.

use titan_common::{ControlResponse, VmWindowRecord};

use super::errors::ServeError;
use super::state::{ServeState, VmWindowReloadMsg};

const MAX_SNAPSHOT_JSON_BYTES: usize = 4 * 1024 * 1024;

fn ack(ok: bool, applied: u32, detail: impl Into<String>) -> ControlResponse {
    ControlResponse::ApplyVmWindowSnapshotAck {
        ok,
        applied,
        detail: detail.into(),
    }
}

fn validate_target(device_id: &str) -> Option<ControlResponse> {
    let own_id = crate::host_device_id::host_device_id_string();
    let target = device_id.trim();
    if !target.is_empty() && target != own_id {
        return Some(ack(
            false,
            0,
            format!("snapshot device_id {target} != host {own_id}"),
        ));
    }
    None
}

fn parse_snapshot(records_json: &str) -> Result<Vec<VmWindowRecord>, ControlResponse> {
    if records_json.len() > MAX_SNAPSHOT_JSON_BYTES {
        return Err(ack(
            false,
            0,
            format!(
                "snapshot {} bytes exceeds {}",
                records_json.len(),
                MAX_SNAPSHOT_JSON_BYTES
            ),
        ));
    }
    serde_json::from_str::<Vec<VmWindowRecord>>(records_json)
        .map_err(|e| ack(false, 0, format!("invalid Vec<VmWindowRecord> JSON: {e}")))
}

pub async fn handle_apply_vm_window_snapshot(
    device_id: String,
    records_json: String,
    state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    if let Some(err) = validate_target(&device_id) {
        return Ok(err);
    }
    let rows = match parse_snapshot(&records_json) {
        Ok(r) => r,
        Err(r) => return Ok(r),
    };
    let applied = rows.len() as u32;
    if let Some(tx) = state.vm_windows_reload_tx.as_ref()
        && let Err(e) = tx.send(VmWindowReloadMsg::Replace { records: rows })
    {
        tracing::warn!(error = %e, "vm window reload send failed");
    }
    Ok(ack(true, applied, String::new()))
}
