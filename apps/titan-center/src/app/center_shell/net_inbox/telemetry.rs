use std::time::Instant;

use crate::app::CenterApp;

impl CenterApp {
    pub(super) fn on_net_host_telemetry(
        &mut self,
        host_key: String,
        session_gen: u64,
        push: titan_common::ControlPush,
    ) -> bool {
        if self
            .telemetry_links
            .get(&host_key)
            .is_none_or(|l| l.session_gen != session_gen)
        {
            return true;
        }
        if matches!(push, titan_common::ControlPush::HostByeNow) {
            self.handle_host_bye_now(&host_key);
            return false;
        }
        self.apply_telemetry_push_and_refresh(host_key, push);
        false
    }

    /// Skips repaint for the 50 ms heartbeat to avoid a 20 Hz forced redraw; the next
    /// user/event-driven repaint covers UI freshness, and `tick_telemetry_staleness` still
    /// fires within the staleness window when heartbeats stop arriving.
    fn apply_telemetry_push_and_refresh(
        &mut self,
        host_key: String,
        push: titan_common::ControlPush,
    ) {
        let is_heartbeat = matches!(push, titan_common::ControlPush::HostHeartbeat { .. });
        let host_key_for_ctl = host_key.clone();
        self.apply_control_push_for_telemetry(host_key, push);
        self.mark_telemetry_timestamp_for_key(&host_key_for_ctl, Instant::now());
        self.last_net_error.clear();
        if !is_heartbeat {
            self.ctx.request_repaint();
        }
    }

    fn handle_host_bye_now(&mut self, host_key: &str) {
        self.mark_telemetry_live_for_key(host_key, false);
        if host_key != Self::endpoint_addr_key(&self.control_addr) {
            return;
        }
        self.force_reconnect_to_control_host();
        self.mark_control_endpoint_offline();
        self.ctx.request_repaint();
    }

    pub(super) fn on_net_telemetry_link_lost(&mut self, host_key: String, session_gen: u64) {
        let stale = self
            .telemetry_links
            .get(&host_key)
            .is_none_or(|l| l.session_gen != session_gen);
        if stale {
            return;
        }
        if let Some(s) = self.fleet_by_endpoint.get_mut(&host_key) {
            s.clear_telemetry();
        }
        self.host_resource_stats.remove(&host_key);
        // Keep the last decoded preview frame to avoid a visible blank/flicker while telemetry
        // reconnects after short network hiccups or endpoint switching.
        self.mark_telemetry_live_for_key(&host_key, false);
        if host_key == Self::endpoint_addr_key(&self.control_addr) {
            self.force_reconnect_to_control_host();
            self.mark_control_endpoint_offline();
        }
        self.ctx.request_repaint();
    }
}
