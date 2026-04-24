//! Background std::thread workers for control-plane requests.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use titan_common::{ControlRequest, ControlResponse, VmSpoofProfile};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::constants::{
    ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS, ADD_HOST_VERIFY_UI_DEADLINE_SECS, TELEMETRY_MAX_CONCURRENT,
};
use super::net_client::{
    capabilities_summary, exchange_one, fetch_desktop_snapshot, fetch_host_resource_snapshot,
    hello_host, read_telemetry_push, telemetry_addr_for_control,
};
use super::net_msg::NetUiMsg;
use super::persist_data::HostEndpoint;
use super::tcp_tune::tune_connected_stream;
use super::{CenterApp, TelemetryLink};

/// Per-host Hello in reachability batch: avoid long OS TCP connect stalls on offline hosts.
const HELLO_REACHABILITY_TIMEOUT: Duration = Duration::from_secs(2);

/// Desktop JPEG fetch can include slow capture + large read; still cap so the cycle thread always finishes.
const DESKTOP_SNAPSHOT_FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Host resource snapshot is smaller; separate cap so one bad host does not stall the whole grid.
const HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT: Duration = Duration::from_secs(8);

/// When a row is **known offline** but has seen caps before, fail fast so the grid round does not burn 20s+8s per dead host.
const DESKTOP_SNAPSHOT_FETCH_TIMEOUT_OFFLINE: Duration = Duration::from_secs(3);
const HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT_OFFLINE: Duration = Duration::from_secs(3);

/// Outer wall per endpoint (desktop + optional resource). Catches stalls where inner `timeout` does not fire.
const PER_HOST_DESKTOP_CYCLE_WALL: Duration = Duration::from_secs(55);

/// Ensures [`NetUiMsg::DesktopFetchCycleDone`] is sent when the desktop snapshot worker exits for any reason
/// (including panic inside `block_on`), so [`CenterApp::desktop_fetch_busy`] cannot stick true forever.
struct DesktopFetchCycleGuard(Sender<NetUiMsg>);

impl Drop for DesktopFetchCycleGuard {
    fn drop(&mut self) {
        let _ = self.0.send(NetUiMsg::DesktopFetchCycleDone);
    }
}

fn run_blocking_net(
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
    run: impl FnOnce(&tokio::runtime::Runtime),
) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(NetUiMsg::Error(format!("tokio runtime: {e}")));
            ctx.request_repaint();
            return;
        }
    };
    run(&rt);
    ctx.request_repaint();
}

impl CenterApp {
    /// Background Hello to validate the manual add-host address and read [`Capabilities::device_id`].
    ///
    /// Uses a **multi-thread** Tokio runtime so `timeout` + TCP connect advance reliably (the generic
    /// `run_blocking_net` path uses `current_thread` and can stall the timer on some platforms).
    pub(super) fn spawn_add_host_verify(&mut self, addr: String) {
        if self.add_host_verify_busy {
            return;
        }
        self.add_host_verify_busy = true;
        self.add_host_verify_session = self.add_host_verify_session.wrapping_add(1);
        let sid = self.add_host_verify_session;
        self.add_host_verify_deadline = Some(
            Instant::now() + Duration::from_secs(ADD_HOST_VERIFY_UI_DEADLINE_SECS),
        );
        self.add_host_dialog_err.clear();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(NetUiMsg::AddHostVerifyDone {
                        session_id: sid,
                        addr,
                        ok: false,
                        device_id: String::new(),
                        caps_summary: String::new(),
                        error: format!("tokio runtime: {e}"),
                    });
                    ctx.request_repaint();
                    return;
                }
            };
            let msg = match rt.block_on(timeout(
                Duration::from_secs(ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS),
                hello_host(&addr),
            )) {
                Ok(Ok(ControlResponse::HelloAck { capabilities })) => {
                    let mut did = capabilities.device_id.trim().to_string();
                    if did.is_empty() {
                        did = HostEndpoint::legacy_device_id_for_addr(&addr);
                    }
                    NetUiMsg::AddHostVerifyDone {
                        session_id: sid,
                        addr,
                        ok: true,
                        device_id: did,
                        caps_summary: capabilities_summary(&capabilities),
                        error: String::new(),
                    }
                }
                Ok(Ok(ControlResponse::ServerError { code, message })) => NetUiMsg::AddHostVerifyDone {
                    session_id: sid,
                    addr,
                    ok: false,
                    device_id: String::new(),
                    caps_summary: String::new(),
                    error: format!("host error {code}: {message}"),
                },
                Ok(Ok(_)) => NetUiMsg::AddHostVerifyDone {
                    session_id: sid,
                    addr,
                    ok: false,
                    device_id: String::new(),
                    caps_summary: String::new(),
                    error: "unexpected control response".into(),
                },
                Ok(Err(e)) => NetUiMsg::AddHostVerifyDone {
                    session_id: sid,
                    addr,
                    ok: false,
                    device_id: String::new(),
                    caps_summary: String::new(),
                    error: e.to_string(),
                },
                Err(_) => NetUiMsg::AddHostVerifyDone {
                    session_id: sid,
                    addr,
                    ok: false,
                    device_id: String::new(),
                    caps_summary: String::new(),
                    error: "timeout".into(),
                },
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }

    /// Sends `Hello` on the control TCP (periodic auto-connect from the app update loop).
    pub(super) fn spawn_hello_session(&mut self) {
        if self.net_busy || self.fleet_busy || self.host_connected {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(hello_host(&addr)) {
                    Ok(ControlResponse::HelloAck { capabilities }) => NetUiMsg::Caps {
                        summary: capabilities_summary(&capabilities),
                    },
                    Ok(ControlResponse::Pong { .. }) => NetUiMsg::Error(
                        "unexpected Pong (expected HelloAck); check host version".into(),
                    ),
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(_) => NetUiMsg::Error("unexpected control response".into()),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_list_vms(&mut self) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(&addr, &ControlRequest::ListVms)) {
                    Ok(ControlResponse::VmList { vms }) => NetUiMsg::VmInventory(vms),
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_spoof_apply(&mut self, dry_run: bool) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        if !self.host_connected {
            self.last_net_error =
                "Host session not ready (Hello + telemetry); wait for auto-connect or check address."
                    .into();
            return;
        }
        let vm = self.spoof_target_vm.trim().to_string();
        if vm.is_empty() {
            self.last_net_error = "Spoof: enter a VM name.".into();
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let profile = VmSpoofProfile {
            dynamic_mac: self.spoof_dynamic_mac,
            disable_checkpoints: self.spoof_disable_checkpoints,
            guest_identity_tag: None,
            ..Default::default()
        };
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let req = ControlRequest::ApplySpoofProfile {
                    vm_name: vm,
                    dry_run,
                    spoof: profile,
                };
                let msg = match rt.block_on(exchange_one(&addr, &req)) {
                    Ok(ControlResponse::SpoofApplyAck {
                        steps_executed,
                        notes,
                        dry_run: dr,
                        ..
                    }) => NetUiMsg::SpoofApply {
                        dry_run: dr,
                        steps: steps_executed,
                        notes,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_stop_vm_group(&mut self, vm_names: Vec<String>) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(
                    &addr,
                    &ControlRequest::StopVmGroup { vm_names },
                )) {
                    Ok(ControlResponse::BatchPowerAck {
                        succeeded,
                        failures,
                    }) => NetUiMsg::BatchStop {
                        succeeded,
                        failures,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_start_vm_group(&mut self, vm_names: Vec<String>) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(
                    &addr,
                    &ControlRequest::StartVmGroup { vm_names },
                )) {
                    Ok(ControlResponse::BatchPowerAck {
                        succeeded,
                        failures,
                    }) => NetUiMsg::BatchStart {
                        succeeded,
                        failures,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    /// One `Hello` per saved device to refresh [`HostEndpoint::last_known_online`] for rows without live telemetry.
    pub(super) fn spawn_reachability_probe_cycle(&mut self) {
        if self.reachability_probe_busy || self.endpoints.is_empty() {
            return;
        }
        self.reachability_probe_busy = true;
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let addrs: Vec<String> = self.endpoints.iter().map(|e| e.addr.clone()).collect();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                rt.block_on(async {
                    let mut set = JoinSet::new();
                    for addr in addrs {
                        let addr = addr.clone();
                        set.spawn(async move {
                            let key = CenterApp::endpoint_addr_key(&addr);
                            let online = match timeout(
                                HELLO_REACHABILITY_TIMEOUT,
                                hello_host(&addr),
                            )
                            .await
                            {
                                Ok(Ok(ControlResponse::HelloAck { .. })) => true,
                                Ok(Ok(_)) => true,
                                Ok(Err(_)) | Err(_) => false,
                            };
                            (key, online)
                        });
                    }
                    while let Some(joined) = set.join_next().await {
                        match joined {
                            Ok((key, online)) => {
                                let _ = tx.send(NetUiMsg::HostReachability {
                                    control_addr: key,
                                    online,
                                });
                            }
                            Err(e) => tracing::warn!(error = %e, "reachability probe task failed"),
                        }
                    }
                    let _ = tx.send(NetUiMsg::ReachabilityProbeCycleDone);
                });
            });
        });
    }

    /// Poll each known host for a downscaled desktop JPEG (background thread; uses [`CenterApp::desktop_fetch_busy`] only — does not take [`CenterApp::net_busy`]).
    pub(super) fn spawn_desktop_snapshot_cycle(&mut self) {
        if self.desktop_fetch_busy || self.endpoints.is_empty() {
            return;
        }
        self.desktop_fetch_busy = true;
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let addrs: Vec<(String, bool, bool)> = self
            .endpoints
            .iter()
            .map(|e| {
                (
                    e.addr.clone(),
                    e.last_known_online,
                    e.last_caps.trim().is_empty(),
                )
            })
            .collect();
        let mut telemetry_live_keys: HashSet<String> = self
            .fleet_by_endpoint
            .iter()
            .filter(|(_, v)| v.telemetry_live)
            .map(|(k, _)| k.clone())
            .collect();
        let primary_key = CenterApp::endpoint_addr_key(&self.control_addr);
        if self.telemetry_live && !primary_key.is_empty() {
            telemetry_live_keys.insert(primary_key.clone());
        }
        std::thread::spawn(move || {
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
            rt.block_on(async move {
                for (idx, (addr, last_known_online, caps_empty)) in addrs.into_iter().enumerate() {
                    let key = CenterApp::endpoint_addr_key(&addr);
                    let skip_duplex_pulls = telemetry_live_keys.contains(&key);
                    let addr_show = addr.clone();
                    let fast_fail_offline = !last_known_online && !caps_empty;
                    let desktop_to = if fast_fail_offline {
                        DESKTOP_SNAPSHOT_FETCH_TIMEOUT_OFFLINE
                    } else {
                        DESKTOP_SNAPSHOT_FETCH_TIMEOUT
                    };
                    let resource_to = if fast_fail_offline {
                        HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT_OFFLINE
                    } else {
                        HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT
                    };
                    match timeout(
                        PER_HOST_DESKTOP_CYCLE_WALL,
                        async {
                            if !skip_duplex_pulls {
                                let snap_res =
                                    timeout(desktop_to, fetch_desktop_snapshot(&addr)).await;
                                match snap_res {
                                    Ok(Ok(ControlResponse::DesktopSnapshotJpeg {
                                        jpeg_bytes,
                                        ..
                                    })) => {
                                        let _ = tx.send(NetUiMsg::DesktopSnapshot {
                                            control_addr: key.clone(),
                                            jpeg_bytes,
                                        });
                                    }
                                    Ok(Ok(ControlResponse::ServerError { code, message })) => {
                                        tracing::warn!(%addr, code, %message, "desktop snapshot rejected by host");
                                    }
                                    Ok(Ok(other)) => {
                                        tracing::warn!(%addr, ?other, "desktop snapshot unexpected response");
                                    }
                                    Ok(Err(e)) => {
                                        tracing::warn!(%addr, %e, "desktop snapshot request failed");
                                    }
                                    Err(_) => {
                                        tracing::warn!(
                                            %addr,
                                            timeout_secs = desktop_to.as_secs(),
                                            "desktop snapshot fetch timed out"
                                        );
                                    }
                                }
                                let res_res = timeout(
                                    resource_to,
                                    fetch_host_resource_snapshot(&addr),
                                )
                                .await;
                                match res_res {
                                    Ok(Ok(ControlResponse::HostResourceSnapshot { stats })) => {
                                        let _ = tx.send(NetUiMsg::HostResources {
                                            control_addr: key.clone(),
                                            stats,
                                        });
                                    }
                                    Ok(Ok(ControlResponse::ServerError { code, message })) => {
                                        tracing::warn!(%addr, code, %message, "host resource snapshot rejected");
                                    }
                                    Ok(Ok(other)) => {
                                        tracing::warn!(%addr, ?other, "host resource snapshot unexpected response");
                                    }
                                    Ok(Err(e)) => {
                                        tracing::warn!(%addr, %e, "host resource snapshot request failed");
                                    }
                                    Err(_) => {
                                        tracing::warn!(
                                            %addr,
                                            timeout_secs = resource_to.as_secs(),
                                            "host resource snapshot fetch timed out"
                                        );
                                    }
                                }
                            }
                        }
                    )
                    .await
                    {
                        Ok(()) => {}
                        Err(_) => {
                            tracing::warn!(
                                idx,
                                addr = %addr_show,
                                wall_secs = PER_HOST_DESKTOP_CYCLE_WALL.as_secs(),
                                "desktop snapshot cycle: per-host wall time exceeded"
                            );
                        }
                    }
                }
            });
            ctx.request_repaint();
        });
    }

    /// Dedicated telemetry TCP for [`Self::control_addr`] (primary session).
    pub(super) fn spawn_telemetry_reader(&mut self) {
        let host_key = CenterApp::endpoint_addr_key(&self.control_addr);
        self.spawn_telemetry_reader_for(host_key, self.control_addr.clone());
    }

    /// One telemetry TCP reader per `host_key` (reconnects with backoff until stopped). Fleet cap: [`TELEMETRY_MAX_CONCURRENT`].
    pub(super) fn spawn_telemetry_reader_for(&mut self, host_key: String, control_addr: String) {
        if host_key.is_empty() || control_addr.trim().is_empty() {
            return;
        }
        let active = self
            .telemetry_links
            .values()
            .filter(|l| l.running.load(Ordering::SeqCst))
            .count();
        let this_running = self
            .telemetry_links
            .get(&host_key)
            .is_some_and(|l| l.running.load(Ordering::SeqCst));
        if !this_running && active >= TELEMETRY_MAX_CONCURRENT {
            let _ = self.net_tx.send(NetUiMsg::Error(format!(
                "telemetry: max {TELEMETRY_MAX_CONCURRENT} concurrent TCP streams (fleet cap)"
            )));
            self.ctx.request_repaint();
            return;
        }
        let telemetry_addr = match telemetry_addr_for_control(&control_addr) {
            Ok(a) => a,
            Err(e) => {
                let _ = self
                    .net_tx
                    .send(NetUiMsg::Error(format!("telemetry address: {e}")));
                self.ctx.request_repaint();
                return;
            }
        };
        let link = self
            .telemetry_links
            .entry(host_key.clone())
            .or_insert_with(|| TelemetryLink {
                session_gen: 0,
                stop: Arc::new(AtomicBool::new(true)),
                running: Arc::new(AtomicBool::new(false)),
            });
        if link.running.load(Ordering::SeqCst) {
            return;
        }
        link.stop = Arc::new(AtomicBool::new(false));
        link.session_gen = link.session_gen.wrapping_add(1);
        let gen = link.session_gen;
        let stop = link.stop.clone();
        let running = link.running.clone();
        running.store(true, Ordering::SeqCst);
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    running.store(false, Ordering::SeqCst);
                    let _ = tx.send(NetUiMsg::Error(format!("telemetry tokio runtime: {e}")));
                    ctx.request_repaint();
                    return;
                }
            };
            rt.block_on(async move {
                let mut backoff_ms: u64 = 200;
                loop {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    match TcpStream::connect(&telemetry_addr).await {
                        Ok(mut sock) => {
                            let _ = tune_connected_stream(&sock);
                            backoff_ms = 200;
                            loop {
                                if stop.load(Ordering::SeqCst) {
                                    break;
                                }
                                match read_telemetry_push(&mut sock).await {
                                    Ok(push) => {
                                        let _ = tx.send(NetUiMsg::HostTelemetry {
                                            host_key: host_key.clone(),
                                            gen,
                                            push,
                                        });
                                        ctx.request_repaint();
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            "telemetry TCP read failed; reconnecting"
                                        );
                                        let _ = tx.send(NetUiMsg::TelemetryLinkLost {
                                            host_key: host_key.clone(),
                                            gen,
                                        });
                                        ctx.request_repaint();
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                addr = %telemetry_addr,
                                error = %e,
                                backoff_ms,
                                "telemetry TCP connect failed; retrying"
                            );
                            tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                            backoff_ms = (backoff_ms.saturating_mul(2)).min(10_000);
                        }
                    }
                }
                running.store(false, Ordering::SeqCst);
            });
        });
    }

    /// Fan-out the same [`ControlRequest`] to many hosts with bounded concurrency (fleet / 群控).
    pub(super) fn spawn_fleet_exchange(
        &mut self,
        req: ControlRequest,
        targets: Vec<(String, String)>,
    ) {
        if self.fleet_busy || targets.is_empty() {
            return;
        }
        self.fleet_busy = true;
        self.last_net_error.clear();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(NetUiMsg::FleetOpResult {
                        host_key: String::new(),
                        ok: false,
                        detail: format!("fleet tokio runtime: {e}"),
                    });
                    let _ = tx.send(NetUiMsg::FleetOpDone);
                    ctx.request_repaint();
                    return;
                }
            };
            rt.block_on(async move {
                let sem = Arc::new(Semaphore::new(32));
                let mut js = JoinSet::new();
                for (host_key, addr) in targets {
                    let tx = tx.clone();
                    let sem = sem.clone();
                    let req = req.clone();
                    js.spawn(async move {
                        let _p = match sem.acquire().await {
                            Ok(p) => p,
                            Err(_) => return,
                        };
                        let (ok, detail) = match exchange_one(&addr, &req).await {
                            Ok(ControlResponse::HelloAck { .. }) => (true, String::new()),
                            Ok(ControlResponse::ServerError { code, message }) => {
                                (false, format!("host error {code}: {message}"))
                            }
                            Ok(other) => (false, format!("unexpected response: {other:?}")),
                            Err(e) => (false, e.to_string()),
                        };
                        let _ = tx.send(NetUiMsg::FleetOpResult {
                            host_key,
                            ok,
                            detail,
                        });
                    });
                }
                while js.join_next().await.is_some() {}
                let _ = tx.send(NetUiMsg::FleetOpDone);
            });
            ctx.request_repaint();
        });
    }

    /// Hello to the device currently selected in device management.
    pub(super) fn spawn_fleet_hello_selected(&mut self) {
        if self.endpoints.is_empty() {
            return;
        }
        let idx = self
            .selected_host
            .min(self.endpoints.len().saturating_sub(1));
        let ep = &self.endpoints[idx];
        let targets = vec![(CenterApp::endpoint_addr_key(&ep.addr), ep.addr.clone())];
        self.spawn_fleet_exchange(ControlRequest::Hello, targets);
    }

    /// Start (or keep) a telemetry TCP reader for the selected device (same cap as [`Self::spawn_telemetry_reader_for`]).
    pub(super) fn spawn_fleet_telemetry_selected(&mut self) {
        if self.endpoints.is_empty() {
            return;
        }
        let idx = self
            .selected_host
            .min(self.endpoints.len().saturating_sub(1));
        let ep = &self.endpoints[idx];
        let host_key = CenterApp::endpoint_addr_key(&ep.addr);
        self.spawn_telemetry_reader_for(host_key, ep.addr.clone());
    }
}
