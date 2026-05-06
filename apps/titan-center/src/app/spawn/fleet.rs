//! Fleet fan-out control requests with bounded concurrency.

use std::sync::mpsc::SyncSender;

use titan_common::{ControlRequest, ControlResponse};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::super::CenterApp;
use super::super::net::{NetUiMsg, exchange_one};

impl CenterApp {
    /// Fan-out the same [`ControlRequest`] to many hosts with bounded concurrency (fleet / 群控).
    pub(crate) fn spawn_fleet_exchange(
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
        std::thread::spawn(move || run_fleet_exchange_worker(tx, ctx, req, targets));
    }

    /// Hello to the device currently selected in device management.
    pub(crate) fn spawn_fleet_hello_selected(&mut self) {
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
    pub(crate) fn spawn_fleet_telemetry_selected(&mut self) {
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

fn run_fleet_exchange_worker(
    tx: SyncSender<NetUiMsg>,
    ctx: egui::Context,
    req: ControlRequest,
    targets: Vec<(String, String)>,
) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            fleet_runtime_error(&tx, &ctx, format!("fleet tokio runtime: {e}"));
            return;
        }
    };
    rt.block_on(run_fleet_joins(tx.clone(), req, targets));
    let _ = tx.send(NetUiMsg::FleetOpDone);
    ctx.request_repaint();
}

fn fleet_runtime_error(tx: &SyncSender<NetUiMsg>, ctx: &egui::Context, detail: String) {
    let _ = tx.send(NetUiMsg::FleetOpResult {
        host_key: String::new(),
        ok: false,
        detail,
    });
    let _ = tx.send(NetUiMsg::FleetOpDone);
    ctx.request_repaint();
}

async fn run_fleet_joins(
    tx: SyncSender<NetUiMsg>,
    req: ControlRequest,
    targets: Vec<(String, String)>,
) {
    let sem = std::sync::Arc::new(Semaphore::new(32));
    let mut js = JoinSet::new();
    for (host_key, addr) in targets {
        spawn_one_fleet_task(&mut js, &tx, &sem, &req, host_key, addr);
    }
    while js.join_next().await.is_some() {}
}

fn spawn_one_fleet_task(
    js: &mut JoinSet<()>,
    tx: &SyncSender<NetUiMsg>,
    sem: &std::sync::Arc<Semaphore>,
    req: &ControlRequest,
    host_key: String,
    addr: String,
) {
    let tx = tx.clone();
    let sem = sem.clone();
    let req = req.clone();
    js.spawn(async move {
        let _p = match sem.acquire().await {
            Ok(p) => p,
            Err(_) => return,
        };
        let (ok, detail) = fleet_one_exchange(&addr, &req).await;
        let _ = tx.send(NetUiMsg::FleetOpResult {
            host_key,
            ok,
            detail,
        });
    });
}

async fn fleet_one_exchange(addr: &str, req: &ControlRequest) -> (bool, String) {
    match exchange_one(addr, req).await {
        Ok(ControlResponse::HelloAck { .. }) => (true, String::new()),
        Ok(ControlResponse::ServerError { code, message }) => {
            (false, format!("host error {code}: {message}"))
        }
        Ok(other) => (false, format!("unexpected response: {other:?}")),
        Err(e) => (false, e.to_string()),
    }
}
