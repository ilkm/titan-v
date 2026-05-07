//! Parallel Hello reachability probe across saved endpoints.

use std::sync::mpsc::SyncSender;
use std::{fs::OpenOptions, io::Write};

use serde_json::json;
use titan_common::ControlResponse;
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::super::CenterApp;
use super::super::net::{NetUiMsg, hello_host};
use super::common::{HELLO_REACHABILITY_TIMEOUT, run_blocking_net};

impl CenterApp {
    /// One `Hello` per saved device to refresh [`HostEndpoint::last_known_online`] for rows without live telemetry.
    pub(crate) fn spawn_reachability_probe_cycle(&mut self) {
        if self.reachability_probe_busy || self.endpoints.is_empty() {
            return;
        }
        self.reachability_probe_busy = true;
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let addrs: Vec<String> = self.endpoints.iter().map(|e| e.addr.clone()).collect();
        std::thread::spawn(move || run_reachability_worker(tx, ctx, addrs));
    }
}

fn run_reachability_worker(tx: SyncSender<NetUiMsg>, ctx: egui::Context, addrs: Vec<String>) {
    run_blocking_net(&tx, &ctx, |rt| {
        rt.block_on(run_reachability_async(tx.clone(), addrs));
    });
}

async fn run_reachability_async(tx: SyncSender<NetUiMsg>, addrs: Vec<String>) {
    let mut set = JoinSet::new();
    for addr in addrs {
        set.spawn(async move { probe_one_addr(addr).await });
    }
    drain_reachability_joins(&tx, &mut set).await;
    let _ = tx.send(NetUiMsg::ReachabilityProbeCycleDone);
}

async fn probe_one_addr(addr: String) -> (String, bool) {
    let key = CenterApp::endpoint_addr_key(&addr);
    let result = timeout(HELLO_REACHABILITY_TIMEOUT, hello_host(&addr)).await;
    let (online, outcome, error) = match result {
        Ok(Ok(ControlResponse::HelloAck { .. })) => (true, "hello_ack", String::new()),
        Ok(Ok(_)) => (true, "non_hello_ack", String::new()),
        Ok(Err(e)) => (false, "hello_err", e.to_string()),
        Err(_) => (false, "hello_timeout", String::new()),
    };
    // #region agent log
    agent_debug_log(
        "H4",
        "spawn/reachability.rs:probe_one_addr",
        "reachability hello result",
        json!({"runId":"run1","addr":addr,"key":key,"online":online,"outcome":outcome,"error":error}),
    );
    // #endregion
    (key, online)
}

async fn drain_reachability_joins(tx: &SyncSender<NetUiMsg>, set: &mut JoinSet<(String, bool)>) {
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
}

fn agent_debug_log(hypothesis_id: &str, location: &str, message: &str, data: serde_json::Value) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    let payload = json!({
        "sessionId":"1f0423",
        "runId":"run1",
        "hypothesisId":hypothesis_id,
        "location":location,
        "message":message,
        "data":data,
        "timestamp":timestamp,
    });
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug-1f0423.log")
    {
        let _ = writeln!(f, "{}", payload);
    }
}
