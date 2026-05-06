//! Shared timeouts and blocking Tokio helper for background threads.

use std::sync::mpsc::SyncSender;
use std::time::Duration;

use super::super::net::NetUiMsg;

/// Per-host Hello in reachability batch: avoid long OS TCP connect stalls on offline hosts.
pub(super) const HELLO_REACHABILITY_TIMEOUT: Duration = Duration::from_secs(2);

/// Desktop JPEG fetch can include slow capture + large read; still cap so the cycle thread always finishes.
pub(super) const DESKTOP_SNAPSHOT_FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Host resource snapshot is smaller; separate cap so one bad host does not stall the whole grid.
pub(super) const HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT: Duration = Duration::from_secs(8);

/// When a row is **known offline** but has seen caps before, fail fast so the grid round does not burn 20s+8s per dead host.
pub(super) const DESKTOP_SNAPSHOT_FETCH_TIMEOUT_OFFLINE: Duration = Duration::from_secs(3);
pub(super) const HOST_RESOURCE_SNAPSHOT_FETCH_TIMEOUT_OFFLINE: Duration = Duration::from_secs(3);

/// Outer wall per endpoint (desktop + optional resource). Catches stalls where inner `timeout` does not fire.
pub(super) const PER_HOST_DESKTOP_CYCLE_WALL: Duration = Duration::from_secs(55);

/// Ensures [`NetUiMsg::DesktopFetchCycleDone`] is sent when the desktop snapshot worker exits for any reason
/// (including panic inside `block_on`), so [`CenterApp::desktop_fetch_busy`] cannot stick true forever.
pub(super) struct DesktopFetchCycleGuard(pub(super) SyncSender<NetUiMsg>);

impl Drop for DesktopFetchCycleGuard {
    fn drop(&mut self) {
        let _ = self.0.send(NetUiMsg::DesktopFetchCycleDone);
    }
}

pub(super) fn run_blocking_net(
    tx: &SyncSender<NetUiMsg>,
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
