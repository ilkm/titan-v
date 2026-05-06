//! `NetUiMsg` dispatch from the UI-thread inbox (`drain_net_inbox`).

mod add_host;
mod fleet_error;
mod host_data;
mod inventory;
mod telemetry;

use crate::app::CenterApp;
use crate::app::net::NetUiMsg;

impl CenterApp {
    pub(crate) fn drain_net_inbox(&mut self) {
        const MAX_MSGS_PER_FRAME: usize = 128;
        for _ in 0..MAX_MSGS_PER_FRAME {
            let Ok(msg) = self.net_rx.try_recv() else {
                return;
            };
            if self.dispatch_net_ui_msg(msg) {
                continue;
            }
        }
        self.ctx.request_repaint();
    }

    /// Returns `true` when the outer `while` should `continue` (stale / ignored message).
    fn dispatch_net_ui_msg(&mut self, msg: NetUiMsg) -> bool {
        if let Some(c) = self.try_net_inventory_ops(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_add_host_verify_only(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_host_resources_desktop(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_reachability_telemetry_ops(&msg) {
            return c;
        }
        self.dispatch_net_fleet_and_error(msg)
    }

    fn try_net_host_resources_desktop(&mut self, msg: &NetUiMsg) -> Option<bool> {
        self.try_net_host_announced_only(msg)
            .or_else(|| self.try_net_host_resources_payload(msg))
    }

    fn try_net_reachability_telemetry_ops(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::HostReachability {
                control_addr,
                online,
            } => {
                self.on_net_host_reachability(control_addr.clone(), *online);
                Some(false)
            }
            NetUiMsg::ReachabilityProbeCycleDone => {
                self.reachability_probe_busy = false;
                Some(false)
            }
            NetUiMsg::HostTelemetry {
                host_key,
                session_gen,
                push,
            } => Some(self.on_net_host_telemetry(host_key.clone(), *session_gen, push.clone())),
            _ => None,
        }
    }
}
