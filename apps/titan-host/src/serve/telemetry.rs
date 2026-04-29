//! Event-driven VM + disk telemetry for the dedicated telemetry TCP plane.

use std::sync::Arc;

use titan_common::{ControlPush, DiskVolume, VmBrief};
use tokio::task;

use super::state::ServeState;

fn disk_volumes_blocking() -> Vec<DiskVolume> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut out = Vec::new();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().into_owned();
        let total = disk.total_space();
        let free = disk.available_space();
        if total > 0 {
            out.push(DiskVolume {
                mount,
                free_bytes: free,
                total_bytes: total,
            });
        }
    }
    out
}

/// Builds a telemetry push. If `reuse_vms` is set (e.g. right after `ListVms`), skips a second VM query.
pub async fn build_telemetry_push(reuse_vms: Option<Vec<VmBrief>>) -> Option<ControlPush> {
    let vms = reuse_vms.unwrap_or_default();

    let volumes: Vec<DiskVolume> = task::spawn_blocking(disk_volumes_blocking)
        .await
        .unwrap_or_default();

    Some(ControlPush::HostTelemetry {
        content_hint: Some(format!("vms={}", vms.len())),
        vms,
        volumes,
    })
}

/// Fire-and-forget telemetry publish after a command response (no center polling).
pub fn publish_telemetry_after_dispatch(
    state: &Arc<ServeState>,
    reuse_vms: Option<Vec<VmBrief>>,
    res: &titan_common::ControlResponse,
) {
    if matches!(res, titan_common::ControlResponse::ServerError { .. }) {
        return;
    }
    let tx = state.telemetry_tx.clone();
    tokio::spawn(async move {
        if let Some(push) = build_telemetry_push(reuse_vms).await {
            let _ = tx.send(push);
        }
    });
}
