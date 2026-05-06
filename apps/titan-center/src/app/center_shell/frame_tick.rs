//! Periodic UI-thread ticks: discovery threads, desktop previews, reachability, telemetry staleness.

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::app::CenterApp;
use crate::app::constants::{
    DESKTOP_PREVIEW_POLL_SECS, REACHABILITY_PROBE_SECS, TELEMETRY_STALE_AFTER_SECS,
};
use crate::app::discovery;
use crate::app::persist_data::NavTab;

impl CenterApp {
    pub(crate) fn prune_discovery_bind_ipv4s_to_scanned_ifaces(&mut self) {
        let valid: HashSet<String> = self
            .discovery_if_rows
            .iter()
            .map(|r| r.ip.to_string())
            .collect();
        self.discovery_bind_ipv4s.retain(|ip| valid.contains(ip));
    }

    pub(crate) fn prune_host_desktop_textures(&mut self) {
        let valid: HashSet<_> = self
            .endpoints
            .iter()
            .map(|e| Self::endpoint_addr_key(&e.addr))
            .collect();
        self.host_desktop_textures.retain(|k, _| valid.contains(k));
        self.host_resource_stats.retain(|k, _| valid.contains(k));
        self.fleet_by_endpoint.retain(|k, _| valid.contains(k));
    }

    pub(crate) fn tick_desktop_preview_refresh(&mut self, ctx: &egui::Context) {
        let on_connect = self.active_nav == NavTab::Connect;
        if self.prev_nav_for_desktop != NavTab::Connect && on_connect {
            self.desktop_poll_accum = DESKTOP_PREVIEW_POLL_SECS;
            self.reachability_poll_accum = REACHABILITY_PROBE_SECS;
        }
        self.prev_nav_for_desktop = self.active_nav;

        if !on_connect {
            self.desktop_poll_accum = 0.0;
            return;
        }
        if self.endpoints.is_empty() {
            self.desktop_poll_accum = 0.0;
            return;
        }
        self.prune_host_desktop_textures();
        self.desktop_poll_accum += ctx.input(|i| i.unstable_dt);
        if self.desktop_poll_accum >= DESKTOP_PREVIEW_POLL_SECS {
            self.desktop_poll_accum = 0.0;
            self.spawn_desktop_snapshot_cycle();
        }
    }

    pub(crate) fn refresh_discovery_iface_rows(&mut self, ui: &egui::Ui) {
        let now = ui.ctx().input(|i| i.time);
        let initial_scan = self.discovery_if_scan_secs < -100_000.0;
        if initial_scan || now - self.discovery_if_scan_secs >= 3.0 {
            self.discovery_if_scan_secs = now;
            self.discovery_if_rows = discovery::list_lan_ipv4_rows();
            self.prune_discovery_bind_ipv4s_to_scanned_ifaces();
        }
    }

    pub(crate) fn tick_discovery_thread(&mut self) {
        let want = self.discovery_broadcast;
        let sig = self.discovery_udp_spawn_sig();
        if want {
            self.discovery_udp_maybe_respawn(sig);
        } else if self.discovery_active_sig.is_some() {
            self.discovery_gen.fetch_add(1, Ordering::SeqCst);
            self.discovery_active_sig = None;
        }
    }

    fn discovery_udp_spawn_sig(&self) -> discovery::DiscoverySpawnSig {
        discovery::DiscoverySpawnSig::new(
            self.discovery_interval_secs.max(1),
            self.discovery_udp_port,
            self.control_addr.clone(),
            self.discovery_bind_ipv4s.clone(),
        )
    }

    fn discovery_udp_maybe_respawn(&mut self, sig: discovery::DiscoverySpawnSig) {
        let need_spawn = self.discovery_active_sig.as_ref() != Some(&sig);
        if !need_spawn {
            return;
        }
        if self.discovery_active_sig.is_some() {
            self.discovery_gen.fetch_add(1, Ordering::SeqCst);
        }
        let my_gen = self.discovery_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let spawn_generation = self.discovery_gen.clone();
        let interval = Duration::from_secs(u64::from(sig.interval_secs));
        let port = sig.port;
        let host_control = sig.host_control.clone();
        let bind = sig.bind_ipv4s.clone();
        std::thread::spawn(move || {
            discovery::discovery_udp_loop(
                my_gen,
                spawn_generation,
                interval,
                port,
                host_control,
                bind,
            );
        });
        self.discovery_active_sig = Some(sig);
    }

    pub(crate) fn tick_host_collect_thread(&mut self) {
        let want = self.host_collect_broadcast;
        let sig = discovery::HostCollectSpawnSig::new(
            self.host_collect_interval_secs.max(1),
            self.host_collect_poll_udp_port,
            self.host_collect_register_udp_port,
            self.discovery_bind_ipv4s.clone(),
        );

        if want {
            let need_spawn = self.host_collect_active_sig.as_ref() != Some(&sig);
            if need_spawn {
                self.spawn_host_collect_udp_thread(&sig);
            }
        } else if self.host_collect_active_sig.is_some() {
            self.host_collect_gen.fetch_add(1, Ordering::SeqCst);
            self.host_collect_active_sig = None;
        }
    }

    fn spawn_host_collect_udp_thread(&mut self, sig: &discovery::HostCollectSpawnSig) {
        if self.host_collect_active_sig.is_some() {
            self.host_collect_gen.fetch_add(1, Ordering::SeqCst);
        }
        let my_gen = self.host_collect_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let spawn_generation = self.host_collect_gen.clone();
        let interval = Duration::from_secs(u64::from(sig.interval_secs));
        let poll_port = sig.poll_port;
        let register_port = sig.register_port;
        let bind = sig.bind_ipv4s.clone();
        std::thread::spawn(move || {
            discovery::center_host_collect_udp_loop(
                my_gen,
                spawn_generation,
                interval,
                poll_port,
                register_port,
                bind,
            );
        });
        self.host_collect_active_sig = Some(sig.clone());
    }

    pub(crate) fn tick_list_vms_auto_refresh(&mut self, ctx: &egui::Context) {
        if !self.is_control_connected() {
            self.list_vms_poll_accum = 0.0;
            return;
        }
        if self.is_control_telemetry_live() {
            self.list_vms_poll_accum = 0.0;
            return;
        }
        if self.list_vms_auto_refresh && !self.net_busy && !self.control_addr.trim().is_empty() {
            self.list_vms_poll_accum += ctx.input(|i| i.unstable_dt);
            if self.list_vms_poll_accum >= self.list_vms_poll_secs.max(5) as f32 {
                self.list_vms_poll_accum = 0.0;
                self.spawn_list_vms();
            }
        }
    }

    pub(crate) fn tick_reachability_probes(&mut self, _ctx: &egui::Context) {
        if self.endpoints.is_empty() {
            self.reachability_poll_accum = 0.0;
            return;
        }
        if self.reachability_probe_busy {
            return;
        }
        let now = Instant::now();
        let dt = now
            .saturating_duration_since(self.reachability_wall_anchor)
            .as_secs_f32()
            .min(30.0);
        self.reachability_wall_anchor = now;
        self.reachability_poll_accum += dt;
        if self.reachability_poll_accum >= REACHABILITY_PROBE_SECS {
            self.reachability_poll_accum = 0.0;
            self.maintain_fleet_telemetry_readers();
            self.spawn_reachability_probe_cycle();
        }
    }

    pub(crate) fn tick_telemetry_staleness(&mut self) {
        let (command_ready, telemetry_live, last_telemetry_at) =
            self.current_control_session_flags();
        if !telemetry_live {
            return;
        }
        let Some(t) = last_telemetry_at else {
            return;
        };
        if t.elapsed() <= Duration::from_secs_f64(TELEMETRY_STALE_AFTER_SECS) {
            return;
        }
        let key = Self::endpoint_addr_key(&self.control_addr);
        self.mark_telemetry_live_for_key(&key, false);
        if command_ready {
            self.force_reconnect_to_control_host();
        }
        self.ctx.request_repaint();
    }

    pub(crate) fn select_endpoint_host(&mut self, index: usize) {
        if index >= self.endpoints.len() {
            return;
        }
        let new = self.endpoints[index].addr.clone();
        let new_key = Self::endpoint_addr_key(&new);
        let old_key = Self::endpoint_addr_key(&self.control_addr);
        let changed = new_key != old_key;
        let has_live_telemetry = self.has_running_telemetry_link_for_addr(&new_key);
        self.selected_host = index;
        self.control_addr = new;
        if changed {
            self.list_vms_poll_accum = 0.0;
            self.last_capabilities.clear();
            self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
        }
        if !has_live_telemetry {
            self.spawn_fleet_telemetry_selected();
        }
    }
}
