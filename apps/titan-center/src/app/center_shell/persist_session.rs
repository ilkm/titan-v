//! Endpoint selection, dual-channel lifecycle, add-host watchdog, and persistence snapshot helpers.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use egui::{pos2, Area, Color32, CornerRadius, Frame, Margin, Order, RichText};

use crate::app::device_store;
use crate::app::i18n::{self, Msg};
use crate::app::persist_data::{CenterPersist, HostEndpoint};
use crate::app::CenterApp;

impl CenterApp {
    pub(crate) fn selected_endpoint_key(&self) -> Option<String> {
        self.endpoints
            .get(self.selected_host)
            .map(|e| Self::endpoint_addr_key(&e.addr))
    }

    /// Prefer per-host fleet VM/disk lists; fall back to legacy single-host fields.
    pub(crate) fn inventory_slice(&self) -> &[titan_common::VmBrief] {
        if let Some(ref k) = self.selected_endpoint_key() {
            if let Some(s) = self.fleet_by_endpoint.get(k) {
                if !s.vms.is_empty() {
                    return s.vms.as_slice();
                }
            }
        }
        &self.vm_inventory
    }

    pub(crate) fn disk_volumes_slice(&self) -> &[titan_common::DiskVolume] {
        if let Some(ref k) = self.selected_endpoint_key() {
            if let Some(s) = self.fleet_by_endpoint.get(k) {
                if !s.volumes.is_empty() {
                    return s.volumes.as_slice();
                }
            }
        }
        &self.host_disk_volumes
    }

    /// Stops telemetry thread, clears session flags (command + telemetry + `host_connected`).
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

    /// When the selected host has a live telemetry session, a periodic Hello failure must not clear the card (telemetry is authoritative).
    pub(crate) fn should_skip_probe_offline_for_addr(&self, addr_key: &str) -> bool {
        Self::endpoint_addr_key(&self.control_addr) == addr_key && self.host_connected
    }

    /// When `control_addr` is set and there is no session, periodically sends `Hello` (no manual Connect).
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

    /// Normalize for comparing persisted / LAN / typed addresses.
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

    /// Manual add-host after a successful Hello: merge by `device_id`, upgrade legacy same-addr row, or append.
    pub(crate) fn merge_add_host_after_verify(
        &mut self,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        let new_key = Self::endpoint_addr_key(&addr);
        if let Some(pos) = self.endpoints.iter().position(|e| e.device_id == device_id) {
            self.merge_add_host_existing_device(pos, &addr, &new_key, caps_summary);
            return;
        }
        if let Some(pos) = self.merge_add_host_legacy_row_index(&new_key) {
            self.apply_merge_add_host_legacy_row(pos, addr, device_id, caps_summary);
            return;
        }
        self.push_new_host_endpoint_after_verify(addr, device_id, caps_summary);
    }

    fn merge_add_host_legacy_row_index(&self, new_key: &str) -> Option<usize> {
        self.endpoints.iter().position(|e| {
            Self::endpoint_addr_key(&e.addr) == new_key
                && (e.device_id.trim().is_empty()
                    || e.device_id == HostEndpoint::legacy_device_id_for_addr(&e.addr))
        })
    }

    fn apply_merge_add_host_legacy_row(
        &mut self,
        pos: usize,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        let ep = &mut self.endpoints[pos];
        ep.device_id = device_id;
        ep.addr = addr;
        ep.last_caps = caps_summary;
        ep.last_known_online = true;
    }

    fn push_new_host_endpoint_after_verify(
        &mut self,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        self.endpoints.push(HostEndpoint {
            label: format!("host-{}", self.endpoints.len() + 1),
            addr,
            device_id,
            remark: String::new(),
            last_caps: caps_summary,
            last_vm_count: 0,
            last_known_online: true,
        });
    }

    fn merge_add_host_existing_device(
        &mut self,
        pos: usize,
        addr: &str,
        new_key: &str,
        caps_summary: String,
    ) {
        let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
        if old_key != *new_key {
            self.stop_telemetry_reader_for_key(&old_key);
            self.remap_host_caches_addr_key(&old_key, new_key);
            if old_key == Self::endpoint_addr_key(&self.control_addr) {
                self.control_addr = addr.to_string();
                self.command_ready = false;
                self.host_connected = false;
                self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
            }
        }
        let ep = &mut self.endpoints[pos];
        ep.addr = addr.to_string();
        ep.last_caps = caps_summary;
        ep.last_known_online = true;
    }

    /// Invalidate an in-flight add-host Hello (cancel, dialog close, or UI watchdog).
    pub(crate) fn invalidate_add_host_probe(&mut self) {
        self.add_host_verify_session = self.add_host_verify_session.wrapping_add(1);
        self.add_host_verify_busy = false;
        self.add_host_verify_deadline = None;
    }

    pub(crate) fn tick_add_host_verify_watchdog(&mut self, ctx: &egui::Context) {
        if !self.add_host_verify_busy {
            return;
        }
        let Some(dl) = self.add_host_verify_deadline else {
            return;
        };
        let now = Instant::now();
        if now < dl {
            let wait = dl
                .saturating_duration_since(now)
                .min(Duration::from_millis(400));
            ctx.request_repaint_after(wait);
            return;
        }
        self.invalidate_add_host_probe();
        self.ui_toast_text = i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
        self.ui_toast_until = Some(ctx.input(|i| i.time) + 3.8);
        ctx.request_repaint();
    }

    pub(crate) fn render_ui_toast(&self, ctx: &egui::Context) {
        let Some(until) = self.ui_toast_until else {
            return;
        };
        let now = ctx.input(|i| i.time);
        if now >= until || self.ui_toast_text.is_empty() {
            return;
        }
        let screen = ctx.screen_rect();
        let p = pos2(screen.center().x - 100.0, screen.max.y - 56.0);
        Area::new(egui::Id::new("titan_center_ui_toast"))
            .order(Order::Foreground)
            .fixed_pos(p)
            .show(ctx, |ui| {
                Frame::NONE
                    .fill(Color32::from_black_alpha(210))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(18, 11))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&self.ui_toast_text)
                                .color(Color32::WHITE)
                                .size(14.0),
                        );
                    });
            });
    }

    /// After Hello/Ping, attach capability text to the row matching [`Self::control_addr`], creating one if needed.
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

    pub(crate) fn persist_snapshot(&self) -> CenterPersist {
        CenterPersist {
            accounts: self.accounts.clone(),
            proxy_labels: self.proxy_labels.clone(),
            last_script_version: self.last_script_version.clone(),
            list_vms_auto_refresh: self.list_vms_auto_refresh,
            list_vms_poll_secs: self.list_vms_poll_secs.max(5),
            discovery_broadcast: self.discovery_broadcast,
            discovery_interval_secs: self.discovery_interval_secs.max(1),
            discovery_udp_port: self.discovery_udp_port,
            discovery_bind_ipv4s: self.discovery_bind_ipv4s.clone(),
            host_collect_broadcast: self.host_collect_broadcast,
            host_collect_interval_secs: self.host_collect_interval_secs.max(1),
            host_collect_poll_udp_port: self.host_collect_poll_udp_port,
            host_collect_register_udp_port: self.host_collect_register_udp_port,
            ui_lang: self.ui_lang,
            active_nav: self.active_nav,
        }
    }

    pub(crate) fn flush_center_settings_to_sqlite(&self) {
        self.persist_registered_devices();
        let snap = self.persist_snapshot();
        let json = match serde_json::to_string(&snap) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("device_store: center persist snapshot serde: {e}");
                return;
            }
        };
        let db_path = device_store::registration_db_path();
        if let Err(e) = device_store::save_center_persist_json(&db_path, &json) {
            tracing::warn!("device_store: center persist {:?}: {e}", db_path);
        }
    }

    pub(crate) fn maybe_flush_center_sqlite(&mut self, ctx: &egui::Context) {
        const PERIOD_SECS: f64 = 10.0;
        let t = ctx.input(|i| i.time);
        if t - self.sqlite_snapshot_last_time < PERIOD_SECS {
            return;
        }
        self.sqlite_snapshot_last_time = t;
        self.flush_center_settings_to_sqlite();
    }
}
