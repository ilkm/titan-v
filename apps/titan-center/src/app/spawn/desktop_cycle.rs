//! Background desktop JPEG + host resource snapshot polling per endpoint.

use std::collections::HashSet;
use std::sync::mpsc::Sender;

use anyhow::Error;
use titan_common::ControlResponse;
use tokio::time::error::Elapsed;
use tokio::time::timeout;

use super::super::net::{fetch_desktop_snapshot, fetch_host_resource_snapshot, NetUiMsg};
use super::super::CenterApp;
use super::common::{
    DesktopFetchCycleGuard, DESKTOP_SNAPSHOT_FETCH_TIMEOUT, DESKTOP_SNAPSHOT_FETCH_TIMEOUT_OFFLINE,
    HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT, HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT_OFFLINE,
    PER_HOST_DESKTOP_CYCLE_WALL,
};

impl CenterApp {
    /// Poll each known host for a downscaled desktop JPEG (background thread; uses [`CenterApp::desktop_fetch_busy`] only — does not take [`CenterApp::net_busy`]).
    pub(crate) fn spawn_desktop_snapshot_cycle(&mut self) {
        if self.desktop_fetch_busy || self.endpoints.is_empty() {
            return;
        }
        self.desktop_fetch_busy = true;
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let addrs = snapshot_addrs_from_endpoints(&self.endpoints);
        let telemetry_live_keys = telemetry_keys_for_cycle(self);
        std::thread::spawn(move || run_desktop_cycle_thread(tx, ctx, addrs, telemetry_live_keys));
    }
}

fn snapshot_addrs_from_endpoints(
    endpoints: &[super::super::persist_data::HostEndpoint],
) -> Vec<(String, bool, bool)> {
    endpoints
        .iter()
        .map(|e| {
            (
                e.addr.clone(),
                e.last_known_online,
                e.last_caps.trim().is_empty(),
            )
        })
        .collect()
}

fn telemetry_keys_for_cycle(app: &CenterApp) -> HashSet<String> {
    let mut keys: HashSet<String> = app
        .fleet_by_endpoint
        .iter()
        .filter(|(_, v)| v.telemetry_live)
        .map(|(k, _)| k.clone())
        .collect();
    let primary_key = CenterApp::endpoint_addr_key(&app.control_addr);
    if app.telemetry_live && !primary_key.is_empty() {
        keys.insert(primary_key);
    }
    keys
}

fn run_desktop_cycle_thread(
    tx: Sender<NetUiMsg>,
    ctx: egui::Context,
    addrs: Vec<(String, bool, bool)>,
    telemetry_live_keys: HashSet<String>,
) {
    let _cycle_done = DesktopFetchCycleGuard(tx.clone());
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "desktop snapshot worker: failed to build multi-thread tokio runtime"
            );
            return;
        }
    };
    rt.block_on(poll_all_desktops(tx, addrs, telemetry_live_keys));
    ctx.request_repaint();
}

async fn poll_all_desktops(
    tx: Sender<NetUiMsg>,
    addrs: Vec<(String, bool, bool)>,
    telemetry_live_keys: HashSet<String>,
) {
    for (idx, (addr, last_known_online, caps_empty)) in addrs.into_iter().enumerate() {
        poll_one_desktop_slot(
            &tx,
            idx,
            addr,
            last_known_online,
            caps_empty,
            &telemetry_live_keys,
        )
        .await;
    }
}

async fn poll_one_desktop_slot(
    tx: &Sender<NetUiMsg>,
    idx: usize,
    addr: String,
    last_known_online: bool,
    caps_empty: bool,
    telemetry_live_keys: &HashSet<String>,
) {
    let key = CenterApp::endpoint_addr_key(&addr);
    let skip_duplex_pulls = telemetry_live_keys.contains(&key);
    let addr_show = addr.clone();
    let fast_fail_offline = !last_known_online && !caps_empty;
    let (desktop_to, resource_to) = desktop_resource_timeouts(fast_fail_offline);
    let res = timeout(
        PER_HOST_DESKTOP_CYCLE_WALL,
        pull_duplex_snapshots(tx, skip_duplex_pulls, &addr, &key, desktop_to, resource_to),
    )
    .await;
    if res.is_err() {
        tracing::warn!(
            idx,
            addr = %addr_show,
            wall_secs = PER_HOST_DESKTOP_CYCLE_WALL.as_secs(),
            "desktop snapshot cycle: per-host wall time exceeded"
        );
    }
}

fn desktop_resource_timeouts(
    fast_fail_offline: bool,
) -> (std::time::Duration, std::time::Duration) {
    (
        pick_timeout(
            fast_fail_offline,
            DESKTOP_SNAPSHOT_FETCH_TIMEOUT_OFFLINE,
            DESKTOP_SNAPSHOT_FETCH_TIMEOUT,
        ),
        pick_timeout(
            fast_fail_offline,
            HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT_OFFLINE,
            HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT,
        ),
    )
}

fn pick_timeout(
    fast: bool,
    offline: std::time::Duration,
    online: std::time::Duration,
) -> std::time::Duration {
    if fast {
        offline
    } else {
        online
    }
}

async fn pull_duplex_snapshots(
    tx: &Sender<NetUiMsg>,
    skip_duplex_pulls: bool,
    addr: &str,
    key: &str,
    desktop_to: std::time::Duration,
    resource_to: std::time::Duration,
) {
    if skip_duplex_pulls {
        return;
    }
    fetch_desktop_into_channel(tx, addr, key, desktop_to).await;
    fetch_resources_into_channel(tx, addr, key, resource_to).await;
}

fn handle_desktop_control_response(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    inner: Result<ControlResponse, Error>,
) {
    match inner {
        Ok(ControlResponse::DesktopSnapshotJpeg { jpeg_bytes, .. }) => {
            let _ = tx.send(NetUiMsg::DesktopSnapshot {
                control_addr: key.to_string(),
                jpeg_bytes,
            });
        }
        Ok(ControlResponse::ServerError { code, message }) => {
            tracing::warn!(%addr, code, %message, "desktop snapshot rejected by host");
        }
        Ok(other) => {
            tracing::warn!(%addr, ?other, "desktop snapshot unexpected response");
        }
        Err(e) => {
            tracing::warn!(%addr, %e, "desktop snapshot request failed");
        }
    }
}

fn handle_desktop_fetch_outcome(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    desktop_to: std::time::Duration,
    snap_res: Result<Result<ControlResponse, Error>, Elapsed>,
) {
    match snap_res {
        Ok(inner) => handle_desktop_control_response(tx, addr, key, inner),
        Err(_) => {
            tracing::warn!(
                %addr,
                timeout_secs = desktop_to.as_secs(),
                "desktop snapshot fetch timed out"
            );
        }
    }
}

async fn fetch_desktop_into_channel(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    desktop_to: std::time::Duration,
) {
    let snap_res = timeout(desktop_to, fetch_desktop_snapshot(addr)).await;
    handle_desktop_fetch_outcome(tx, addr, key, desktop_to, snap_res);
}

fn handle_resource_control_response(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    inner: Result<ControlResponse, Error>,
) {
    match inner {
        Ok(ControlResponse::HostResourceSnapshot { stats }) => {
            let _ = tx.send(NetUiMsg::HostResources {
                control_addr: key.to_string(),
                stats,
            });
        }
        Ok(ControlResponse::ServerError { code, message }) => {
            tracing::warn!(%addr, code, %message, "host resource snapshot rejected");
        }
        Ok(other) => {
            tracing::warn!(%addr, ?other, "host resource snapshot unexpected response");
        }
        Err(e) => {
            tracing::warn!(%addr, %e, "host resource snapshot request failed");
        }
    }
}

fn handle_resource_fetch_outcome(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    resource_to: std::time::Duration,
    res_res: Result<Result<ControlResponse, Error>, Elapsed>,
) {
    match res_res {
        Ok(inner) => handle_resource_control_response(tx, addr, key, inner),
        Err(_) => {
            tracing::warn!(
                %addr,
                timeout_secs = resource_to.as_secs(),
                "host resource snapshot fetch timed out"
            );
        }
    }
}

async fn fetch_resources_into_channel(
    tx: &Sender<NetUiMsg>,
    addr: &str,
    key: &str,
    resource_to: std::time::Duration,
) {
    let res_res = timeout(resource_to, fetch_host_resource_snapshot(addr)).await;
    handle_resource_fetch_outcome(tx, addr, key, resource_to, res_res);
}
