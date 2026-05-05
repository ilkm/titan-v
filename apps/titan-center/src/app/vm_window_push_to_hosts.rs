//! Center → Host: fan-out [`titan_common::ControlRequest::ApplyVmWindowSnapshot`].
//!
//! Center is the sole source of truth (Center-side SQLite). After every successful CRUD on the
//! local DB, Center pushes the *device-filtered* authoritative list to the affected host (or to
//! all hosts when `device_id` is empty / multiple endpoints share the same id). Hosts replace
//! their in-memory list and the read-only `panel_window_mgmt` viewer redraws on the next frame.
//!
//! All sends are fire-and-forget: a fresh TCP connection is opened per push (mirrors the existing
//! [`crate::app::net::client::exchange_one`] style) and failures are logged but do not block the
//! UI thread. Real-time guarantees come from "every mutation triggers a push" rather than from a
//! long-lived subscription channel.

use anyhow::Context;
use titan_common::{ControlRequest, ControlResponse, VmWindowRecord};

use crate::app::net::exchange_one;
use crate::app::persist_data::HostEndpoint;

fn build_blocking_rt() -> anyhow::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("tokio current_thread runtime build")
}

fn rows_for_device(rows: &[VmWindowRecord], device_id: &str) -> Vec<VmWindowRecord> {
    rows.iter()
        .filter(|r| r.device_id.trim() == device_id)
        .cloned()
        .collect()
}

fn endpoints_for_device(eps: &[HostEndpoint], device_id: &str) -> Vec<HostEndpoint> {
    eps.iter()
        .filter(|e| !e.device_id.trim().is_empty() && e.device_id.trim() == device_id)
        .cloned()
        .collect()
}

fn snapshot_request(device_id: &str, rows: &[VmWindowRecord]) -> ControlRequest {
    let json = serde_json::to_string(rows).unwrap_or_else(|_| "[]".to_string());
    ControlRequest::ApplyVmWindowSnapshot {
        device_id: device_id.to_string(),
        records_json: json,
    }
}

/// Blocking helper used by tests; production code prefers [`push_snapshot_for_device`] which
/// spawns the work onto a worker thread to keep the UI responsive.
pub fn tcp_apply_snapshot(addr: &str, req: &ControlRequest) -> anyhow::Result<()> {
    let rt = build_blocking_rt()?;
    let res = rt.block_on(exchange_one(addr, req))?;
    match res {
        ControlResponse::ApplyVmWindowSnapshotAck { ok: true, .. } => Ok(()),
        ControlResponse::ApplyVmWindowSnapshotAck { detail, .. } => {
            Err(anyhow::anyhow!("host rejected snapshot: {detail}"))
        }
        ControlResponse::ServerError { message, .. } => Err(anyhow::anyhow!("host: {message}")),
        _ => Err(anyhow::anyhow!(
            "unexpected response for ApplyVmWindowSnapshot"
        )),
    }
}

fn spawn_push(addr: String, req: ControlRequest) {
    std::thread::spawn(move || {
        if let Err(e) = tcp_apply_snapshot(&addr, &req) {
            tracing::warn!(error = %e, %addr, "ApplyVmWindowSnapshot push failed");
        }
    });
}

/// Push the snapshot rows for `device_id` to every endpoint matching that device id.
///
/// Useful after a CRUD on a single host's rows: only that host needs the new view. When no
/// endpoint matches (host offline or unknown), the call is a no-op.
pub fn push_snapshot_for_device(
    endpoints: &[HostEndpoint],
    all_rows: &[VmWindowRecord],
    device_id: &str,
) {
    let did = device_id.trim();
    if did.is_empty() {
        return;
    }
    let targets = endpoints_for_device(endpoints, did);
    if targets.is_empty() {
        return;
    }
    let filtered = rows_for_device(all_rows, did);
    let req = snapshot_request(did, &filtered);
    for ep in targets {
        spawn_push(ep.addr, req.clone());
    }
}

/// Push every host its own filtered snapshot (used on bulk reload / startup).
pub fn push_snapshot_to_all(endpoints: &[HostEndpoint], all_rows: &[VmWindowRecord]) {
    for ep in endpoints {
        let did = ep.device_id.trim();
        if did.is_empty() {
            continue;
        }
        let filtered = rows_for_device(all_rows, did);
        let req = snapshot_request(did, &filtered);
        spawn_push(ep.addr.clone(), req);
    }
}

/// Push the snapshot to a single endpoint by addr (initial sync after an endpoint comes online).
pub fn push_snapshot_to_endpoint(endpoint: &HostEndpoint, all_rows: &[VmWindowRecord]) {
    let did = endpoint.device_id.trim();
    if did.is_empty() {
        return;
    }
    let filtered = rows_for_device(all_rows, did);
    let req = snapshot_request(did, &filtered);
    spawn_push(endpoint.addr.clone(), req);
}
