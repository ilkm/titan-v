//! Parallel Hello reachability probe across saved endpoints.

use std::sync::mpsc::SyncSender;

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
    let online = match timeout(HELLO_REACHABILITY_TIMEOUT, hello_host(&addr)).await {
        Ok(Ok(ControlResponse::HelloAck { .. })) => true,
        Ok(Ok(_)) => true,
        Ok(Err(_)) | Err(_) => false,
    };
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
