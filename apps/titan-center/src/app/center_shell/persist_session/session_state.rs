use std::sync::atomic::Ordering;

use crate::app::CenterApp;
use crate::app::persist_data::HostEndpoint;

impl CenterApp {
    pub(crate) fn current_control_session_flags(&self) -> (bool, bool, Option<std::time::Instant>) {
        self.selected_host_session()
            .map(|s| (s.command_ready, s.telemetry_live, s.last_telemetry_at))
            .unwrap_or((false, false, None))
    }

    pub(crate) fn is_control_connected(&self) -> bool {
        let (command_ready, telemetry_live, _) = self.current_control_session_flags();
        command_ready && telemetry_live
    }

    pub(crate) fn is_control_telemetry_live(&self) -> bool {
        let (_, telemetry_live, _) = self.current_control_session_flags();
        telemetry_live
    }

    fn selected_host_session(&self) -> Option<&crate::app::HostControlSession> {
        let key = Self::endpoint_addr_key(&self.control_addr);
        (!key.is_empty())
            .then_some(key)
            .and_then(|k| self.host_sessions.get(&k))
    }

    pub(crate) fn mark_command_ready_for_addr(&mut self, addr: &str, ready: bool) {
        let key = Self::endpoint_addr_key(addr);
        if key.is_empty() {
            return;
        }
        self.host_sessions.entry(key).or_default().command_ready = ready;
    }

    pub(crate) fn mark_telemetry_live_for_key(&mut self, host_key: &str, live: bool) {
        if host_key.is_empty() {
            return;
        }
        let s = self.host_sessions.entry(host_key.to_string()).or_default();
        s.telemetry_live = live;
        if !live {
            s.last_telemetry_at = None;
        }
    }

    pub(crate) fn mark_telemetry_timestamp_for_key(
        &mut self,
        host_key: &str,
        at: std::time::Instant,
    ) {
        if host_key.is_empty() {
            return;
        }
        let s = self.host_sessions.entry(host_key.to_string()).or_default();
        s.telemetry_live = true;
        s.last_telemetry_at = Some(at);
    }

    pub(crate) fn selected_endpoint_key(&self) -> Option<String> {
        self.endpoints
            .get(self.selected_host)
            .map(|e| Self::endpoint_addr_key(&e.addr))
    }

    pub(crate) fn inventory_slice(&self) -> &[titan_common::VmBrief] {
        if let Some(k) = self.selected_endpoint_key()
            && let Some(s) = self.fleet_by_endpoint.get(&k)
            && !s.vms.is_empty()
        {
            return s.vms.as_slice();
        }
        &self.vm_inventory
    }

    pub(crate) fn stop_dual_channels(&mut self) {
        for link in self.telemetry_links.values() {
            link.stop.store(true, Ordering::SeqCst);
        }
        self.telemetry_links.clear();
        self.host_sessions.clear();
        self.auto_hello_accum = 0.0;
        for s in self.fleet_by_endpoint.values_mut() {
            s.clear_telemetry();
        }
    }

    pub(crate) fn mark_control_endpoint_offline(&mut self) {
        let key = Self::endpoint_addr_key(&self.control_addr);
        if key.is_empty() {
            return;
        }
        if let Some(ep) = self
            .endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == key)
        {
            ep.last_known_online = false;
        }
    }

    pub(crate) fn force_reconnect_to_control_host(&mut self) {
        if self.control_addr.trim().is_empty() {
            return;
        }
        let control_addr = self.control_addr.clone();
        self.mark_command_ready_for_addr(&control_addr, false);
        self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
    }

    pub(crate) fn should_skip_probe_offline_for_addr(&self, addr_key: &str) -> bool {
        Self::endpoint_addr_key(&self.control_addr) == addr_key
            && self
                .host_sessions
                .get(addr_key)
                .is_some_and(|s| s.command_ready && s.telemetry_live)
    }

    /// Reachability probes are best-effort and can transiently fail on busy hosts/NAT. If a host
    /// already has a running telemetry reader, keep it online and let telemetry loss decide.
    pub(crate) fn has_running_telemetry_link_for_addr(&self, addr_key: &str) -> bool {
        self.telemetry_links
            .get(addr_key)
            .is_some_and(|l| l.running.load(Ordering::SeqCst) && !l.stop.load(Ordering::SeqCst))
    }

    pub(crate) fn tick_auto_control_session(&mut self, ctx: &egui::Context) {
        if self.control_addr.trim().is_empty() {
            self.auto_hello_accum = 0.0;
            return;
        }
        let (command_ready, telemetry_live, _) = self.current_control_session_flags();
        if (command_ready && telemetry_live) || self.net_busy {
            return;
        }
        if command_ready && !telemetry_live {
            return;
        }
        self.auto_hello_accum += ctx.input(|i| i.unstable_dt);
        if self.auto_hello_accum < Self::AUTO_HELLO_RETRY_SECS {
            return;
        }
        self.auto_hello_accum = 0.0;
        self.spawn_hello_session();
    }

    pub(crate) fn endpoint_addr_key(addr: &str) -> String {
        addr.trim().to_string()
    }

    pub(crate) fn stop_telemetry_reader_for_key(&mut self, host_key: &str) {
        if let Some(link) = self.telemetry_links.remove(host_key) {
            link.stop.store(true, Ordering::SeqCst);
        }
        self.mark_telemetry_live_for_key(host_key, false);
        if let Some(s) = self.fleet_by_endpoint.get_mut(host_key) {
            s.clear_telemetry();
        }
    }

    pub(crate) fn remap_host_caches_addr_key(&mut self, old_key: &str, new_key: &str) {
        if old_key == new_key {
            return;
        }
        if let Some(v) = self.host_sessions.remove(old_key) {
            self.host_sessions.insert(new_key.to_string(), v);
        }
        if let Some(v) = self.fleet_by_endpoint.remove(old_key) {
            self.fleet_by_endpoint.insert(new_key.to_string(), v);
        }
        if let Some(v) = self.host_resource_stats.remove(old_key) {
            self.host_resource_stats.insert(new_key.to_string(), v);
        }
        if let Some(v) = self.host_desktop_textures.remove(old_key) {
            self.host_desktop_textures.insert(new_key.to_string(), v);
        }
    }

    pub(crate) fn upsert_endpoint_after_caps(&mut self, summary: String) {
        let addr = Self::endpoint_addr_key(&self.control_addr);
        self.last_capabilities = summary.clone();
        if addr.is_empty() {
            return;
        }
        if let Some(i) = self
            .endpoints
            .iter()
            .position(|e| Self::endpoint_addr_key(&e.addr) == addr)
        {
            self.endpoints[i].last_caps = summary;
            self.endpoints[i].last_known_online = true;
            self.selected_host = i;
            return;
        }
        let label = format!("host-{}", addr.replace([':', '.', '[', ']'], "-"));
        self.endpoints.push(HostEndpoint {
            label,
            addr: addr.clone(),
            device_id: HostEndpoint::legacy_device_id_for_addr(&addr),
            remark: String::new(),
            last_caps: summary,
            last_vm_count: 0,
            last_known_online: true,
        });
        self.selected_host = self.endpoints.len().saturating_sub(1);
    }

    pub(crate) fn endpoint_mut_for_control_addr(&mut self) -> Option<&mut HostEndpoint> {
        let key = Self::endpoint_addr_key(&self.control_addr);
        if key.is_empty() {
            return None;
        }
        self.endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == key)
    }
}
