use titan_common::{ControlResponse, MAX_PAYLOAD_BYTES};

use crate::ui_persist::HostUiPersist;

use super::errors::ServeError;
use super::response::server_err;
use super::state::ServeState;

const MAX_JSON_BYTES: usize = (MAX_PAYLOAD_BYTES as usize).saturating_sub(48 * 1024);

fn reject_oversized_json(json: &str) -> Option<ControlResponse> {
    if json.len() > MAX_JSON_BYTES {
        return Some(server_err(
            413,
            format!(
                "ApplyHostUiPersistJson body {} bytes exceeds safe limit {}",
                json.len(),
                MAX_JSON_BYTES
            ),
        ));
    }
    None
}

fn parse_and_validate_persist(json: &str) -> Result<HostUiPersist, ControlResponse> {
    let p: HostUiPersist = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => return Err(server_err(400, format!("invalid HostUiPersist JSON: {e}"))),
    };
    if let Err(e) = p.validate_for_remote_apply() {
        return Err(server_err(400, e));
    }
    if let Err(e) = HostUiPersist::bindings_spec(&p) {
        return Err(server_err(400, e));
    }
    Ok(p)
}

pub async fn handle_apply_host_ui_persist_json(
    json: String,
    state: &ServeState,
) -> Result<ControlResponse, ServeError> {
    if let Some(r) = reject_oversized_json(&json) {
        return Ok(r);
    }
    let p = match parse_and_validate_persist(&json) {
        Ok(v) => v,
        Err(r) => return Ok(r),
    };
    let Some(tx) = state.persist_apply_tx.as_ref() else {
        return Ok(server_err(
            503,
            "remote HostUiPersist apply is not enabled on this process",
        ));
    };
    if tx.send(p).is_err() {
        return Ok(server_err(
            503,
            "host UI channel closed; cannot apply remote config",
        ));
    }
    Ok(ControlResponse::HostUiPersistAck {
        ok: true,
        detail: "queued; Host UI will apply on next frame and restart serve".into(),
    })
}
