use std::sync::atomic::Ordering;

use crate::app::CenterApp;
use crate::app::persist_data::HostEndpoint;

impl CenterApp {
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
        self.command_ready = false;
        self.telemetry_live = false;
        self.last_host_telemetry_at = None;
        self.host_connected = false;
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
        self.command_ready = false;
        self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
    }

    pub(crate) fn should_skip_probe_offline_for_addr(&self, addr_key: &str) -> bool {
        Self::endpoint_addr_key(&self.control_addr) == addr_key && self.host_connected
    }

    pub(crate) fn tick_auto_control_session(&mut self, ctx: &egui::Context) {
        if self.control_addr.trim().is_empty() {
            self.auto_hello_accum = 0.0;
            return;
        }
        if self.host_connected || self.net_busy || self.fleet_busy {
            return;
        }
        if self.command_ready && !self.telemetry_live {
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
        if let Some(s) = self.fleet_by_endpoint.get_mut(host_key) {
            s.clear_telemetry();
        }
        if host_key == Self::endpoint_addr_key(&self.control_addr) {
            self.telemetry_live = false;
            self.last_host_telemetry_at = None;
            self.recompute_host_connected();
        }
    }

    pub(crate) fn remap_host_caches_addr_key(&mut self, old_key: &str, new_key: &str) {
        if old_key == new_key {
            return;
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
