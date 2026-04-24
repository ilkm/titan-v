//! Center UI: persisted host table, control-plane (multi-message TCP), VM inventory, scaled grids.

mod constants;
mod device_store;
mod discovery;
mod fleet_state;
mod i18n;
mod lan_host_register;
mod net_client;
mod net_msg;
mod panels_control;
mod panels_danger;
mod panels_hosts;
mod panels_inventory;
mod panels_misc;
mod panels_monitor;
mod panels_spoof;
mod persist_data;
mod spawn;
mod tcp_tune;
mod theme;
mod widgets;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{
    pos2, Align, Area, Color32, CornerRadius, Frame, Layout, Margin, Order, RichText, ScrollArea,
    Sense, Stroke, TextStyle, TextWrapMode, WidgetText,
};
use titan_common::HostResourceStats;

pub use persist_data::{CenterPersist, HostEndpoint, NavTab};

use self::constants::{
    card_shadow, ACCENT, CONTENT_MAX_WIDTH,
    DESKTOP_PREVIEW_POLL_SECS, NAV_ITEM_HEIGHT, PERSIST_KEY, REACHABILITY_PROBE_SECS,
    SIDEBAR_DEFAULT_WIDTH, TELEMETRY_STALE_AFTER_SECS,
};
use self::fleet_state::HostLiveState;
use self::i18n::{Msg, UiLang};
use self::net_msg::NetUiMsg;
use self::persist_data::{
    default_discovery_interval_secs, default_discovery_udp_port,
    default_host_collect_interval_secs, default_list_vms_poll_secs,
};
use self::theme::apply_center_theme;

/// Main column width: uses nearly all space when the window is small; scales up on large monitors.
fn effective_content_width(full_w: f32) -> f32 {
    let scalable = (CONTENT_MAX_WIDTH * 1.15).max(full_w * 0.92);
    full_w.min(scalable).max(280.0)
}

/// Center manager application state (UI thread).
/// One background telemetry TCP session for a given endpoint key ([`CenterApp::endpoint_addr_key`]).
pub(crate) struct TelemetryLink {
    /// Generation for [`NetUiMsg::HostTelemetry::gen`] / [`NetUiMsg::TelemetryLinkLost::gen`]; bumps on stop or new reader.
    pub(crate) session_gen: u64,
    pub(crate) stop: Arc<AtomicBool>,
    pub(crate) running: Arc<AtomicBool>,
}

pub struct CenterApp {
    pub(crate) ctx: egui::Context,
    pub(crate) endpoints: Vec<HostEndpoint>,
    pub(crate) selected_host: usize,
    pub(crate) accounts: Vec<String>,
    pub(crate) proxy_labels: Vec<String>,
    /// Per-endpoint inventory / telemetry (fleet console).
    pub(crate) fleet_by_endpoint: HashMap<String, HostLiveState>,
    pub(crate) fleet_busy: bool,
    pub(crate) vm_inventory: Vec<titan_common::VmBrief>,
    pub(crate) bulk_vm_names: String,
    pub(crate) pending_confirm_stop: bool,
    pub(crate) pending_confirm_start: bool,
    pub(crate) last_action: String,
    pub(crate) control_addr: String,
    pub(crate) net_tx: Sender<NetUiMsg>,
    pub(crate) net_rx: Receiver<NetUiMsg>,
    pub(crate) net_busy: bool,
    pub(crate) host_connected: bool,
    /// Command plane (Hello/Ping) has received a capability-bearing response.
    pub(crate) command_ready: bool,
    /// Telemetry plane has delivered at least one push in this session.
    pub(crate) telemetry_live: bool,
    /// Wall clock when a matching-gen [`NetUiMsg::HostTelemetry`] last arrived (not egui paint time).
    pub(crate) last_host_telemetry_at: Option<Instant>,
    /// Wall clock anchor for [`Self::tick_reachability_probes`] (UI may not repaint when backgrounded).
    pub(crate) reachability_wall_anchor: Instant,
    /// Per-host telemetry TCP readers (bounded by [`constants::TELEMETRY_MAX_CONCURRENT`]).
    pub(crate) telemetry_links: HashMap<String, TelemetryLink>,
    pub(crate) host_disk_volumes: Vec<titan_common::DiskVolume>,
    pub(crate) last_capabilities: String,
    pub(crate) last_net_error: String,
    pub(crate) last_script_version: String,
    pub(crate) list_vms_auto_refresh: bool,
    pub(crate) list_vms_poll_secs: u32,
    pub(crate) list_vms_poll_accum: f32,
    /// Seconds accumulated toward the next automatic `Hello` when disconnected (see `AUTO_HELLO_RETRY_SECS` on `CenterApp`).
    pub(crate) auto_hello_accum: f32,
    /// Accumulator for desktop preview polling on the Connect (device management) tab.
    pub(crate) desktop_poll_accum: f32,
    /// Last nav tab (for detecting entry into Connect → immediate desktop poll).
    pub(crate) prev_nav_for_desktop: NavTab,
    /// Background desktop snapshot cycle in flight (separate from [`Self::net_busy`] so ListVms does not starve previews).
    pub(crate) desktop_fetch_busy: bool,
    /// Accumulator toward periodic Hello probes for every saved device (historical rows / non-telemetry hosts).
    pub(crate) reachability_poll_accum: f32,
    pub(crate) reachability_probe_busy: bool,
    /// Latest decoded desktop preview per host address key ([`Self::endpoint_addr_key`]).
    pub(crate) host_desktop_textures: HashMap<String, egui::TextureHandle>,
    /// Latest [`HostResourceStats`] from host snapshot RPC (same keys as desktop textures).
    pub(crate) host_resource_stats: HashMap<String, HostResourceStats>,
    pub(crate) discovery_gen: Arc<AtomicU64>,
    /// When `Some`, a discovery UDP thread is expected to match this signature.
    pub(crate) discovery_active_sig: Option<discovery::DiscoverySpawnSig>,
    pub(crate) discovery_broadcast: bool,
    pub(crate) discovery_interval_secs: u32,
    pub(crate) discovery_udp_port: u16,
    /// IPv4s to bind for LAN discovery (empty = OS default).
    pub(crate) discovery_bind_ipv4s: Vec<String>,
    pub(crate) host_collect_gen: Arc<AtomicU64>,
    pub(crate) host_collect_active_sig: Option<discovery::HostCollectSpawnSig>,
    pub(crate) host_collect_broadcast: bool,
    pub(crate) host_collect_interval_secs: u32,
    pub(crate) host_collect_poll_udp_port: u16,
    pub(crate) host_collect_register_udp_port: u16,
    pub(crate) discovery_if_rows: Vec<discovery::LanIpv4Row>,
    pub(crate) discovery_if_scan_secs: f64,
    pub(crate) spoof_target_vm: String,
    pub(crate) spoof_dynamic_mac: bool,
    pub(crate) spoof_disable_checkpoints: bool,
    pub(crate) pending_spoof_confirm_apply: bool,
    pub(crate) ui_lang: UiLang,
    pub(crate) settings_open: bool,
    /// Device tab: manual host entry (IP + port), not persisted until saved with app state.
    pub(crate) add_host_dialog_open: bool,
    pub(crate) add_host_dialog_ip: String,
    pub(crate) add_host_dialog_port: String,
    pub(crate) add_host_dialog_err: String,
    pub(crate) add_host_verify_busy: bool,
    /// Bumped when starting a probe, cancelling, closing the dialog, or UI watchdog; stale workers must not apply.
    pub(crate) add_host_verify_session: u64,
    pub(crate) add_host_verify_deadline: Option<Instant>,
    pub(crate) ui_toast_until: Option<f64>,
    pub(crate) ui_toast_text: String,
    pub(crate) active_nav: NavTab,
    /// After tray "Quit", do not cancel the next window close.
    pub(crate) really_quitting: bool,
    /// Main window was hidden to the tray; used to keep egui repainting in the background.
    pub(crate) hidden_to_tray: bool,
    /// Owns the tray icon on Windows/macOS (Linux uses `titan_tray::spawn_linux_tray_thread`).
    pub(crate) _tray: Option<titan_tray::TrayIcon>,
    /// macOS/Winit: tray must be created after the event loop has started (`StartCause::Init`); see tray-icon docs.
    tray_icon_init_attempted: bool,
    /// Device card: index into `endpoints` whose remark is being edited (`None` = display mode).
    pub(crate) device_remark_edit_index: Option<usize>,
    /// Request focus on the remark `TextEdit` the first frame after opening edit mode.
    device_remark_edit_focus_next: bool,
    /// Last painted card height per control addr key (Connect tab masonry / waterfall).
    pub(crate) device_masonry_heights: HashMap<String, f32>,
}

impl CenterApp {
    /// Interval between automatic `Hello` attempts when not connected (`control_addr` non-empty).
    const AUTO_HELLO_RETRY_SECS: f32 = 3.0;

    /// Build the status tray once the winit/eframe loop is running (required on macOS).
    fn maybe_init_tray_icon_once(&mut self) {
        if self.tray_icon_init_attempted {
            return;
        }
        self.tray_icon_init_attempted = true;

        #[cfg(target_os = "linux")]
        {
            return;
        }

        #[cfg(not(target_os = "linux"))]
        match titan_tray::build_tray_icon() {
            Ok(t) => self._tray = Some(t),
            Err(e) => tracing::warn!("system tray unavailable: {e}"),
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_center_theme(&cc.egui_ctx);
        let (net_tx, net_rx) = mpsc::channel();
        let json_opt = cc.storage.and_then(|s| s.get_string(PERSIST_KEY));
        let legacy_eps = json_opt
            .as_deref()
            .and_then(device_store::legacy_endpoints_from_center_json);
        let persist: CenterPersist = json_opt
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_else(|| CenterPersist {
                accounts: vec!["demo-account-1".into()],
                proxy_labels: vec!["proxy-pool-a".into()],
                last_script_version: String::new(),
                list_vms_auto_refresh: false,
                list_vms_poll_secs: default_list_vms_poll_secs(),
                discovery_broadcast: false,
                discovery_interval_secs: default_discovery_interval_secs(),
                discovery_udp_port: default_discovery_udp_port(),
                discovery_bind_ipv4s: Vec::new(),
                host_collect_broadcast: true,
                host_collect_interval_secs: default_host_collect_interval_secs(),
                host_collect_poll_udp_port: titan_common::DEFAULT_CENTER_POLL_UDP_PORT,
                host_collect_register_udp_port: titan_common::DEFAULT_CENTER_REGISTER_UDP_PORT,
                ui_lang: UiLang::default(),
                active_nav: NavTab::default(),
            });
        lan_host_register::spawn_center_lan_host_register_listener(
            net_tx.clone(),
            cc.egui_ctx.clone(),
            persist.host_collect_register_udp_port.max(1),
        );
        let db_path = device_store::registration_db_path();
        let mut endpoints = device_store::load_registered_devices(&db_path).unwrap_or_else(|e| {
            tracing::warn!("device_store: load {:?}: {e}", db_path);
            Vec::new()
        });
        if endpoints.is_empty() {
            if let Some(import) = legacy_eps.filter(|e| !e.is_empty()) {
                endpoints = import;
                if let Err(e) = device_store::save_registered_devices(&db_path, &endpoints) {
                    tracing::warn!("device_store: migrate save {:?}: {e}", db_path);
                }
            }
        }
        for ep in &mut endpoints {
            ep.ensure_device_id();
        }
        let control_addr = endpoints
            .first()
            .map(|e| e.addr.clone())
            .unwrap_or_default();
        let ui_lang = persist.ui_lang;
        let active_nav = persist.active_nav;
        let desktop_poll_accum = if active_nav == NavTab::Connect {
            DESKTOP_PREVIEW_POLL_SECS
        } else {
            0.0
        };
        let auto_hello_accum = if control_addr.trim().is_empty() {
            0.0
        } else {
            Self::AUTO_HELLO_RETRY_SECS
        };
        Self {
            ctx: cc.egui_ctx.clone(),
            endpoints,
            selected_host: 0,
            accounts: persist.accounts,
            proxy_labels: persist.proxy_labels,
            last_script_version: persist.last_script_version,
            list_vms_auto_refresh: persist.list_vms_auto_refresh,
            list_vms_poll_secs: persist.list_vms_poll_secs.max(5),
            list_vms_poll_accum: 0.0,
            auto_hello_accum,
            desktop_poll_accum,
            prev_nav_for_desktop: active_nav,
            desktop_fetch_busy: false,
            reachability_poll_accum: REACHABILITY_PROBE_SECS,
            reachability_probe_busy: false,
            host_desktop_textures: HashMap::new(),
            host_resource_stats: HashMap::new(),
            discovery_gen: Arc::new(AtomicU64::new(0)),
            discovery_active_sig: None,
            discovery_broadcast: persist.discovery_broadcast,
            discovery_interval_secs: persist.discovery_interval_secs.max(1),
            discovery_udp_port: persist.discovery_udp_port,
            discovery_bind_ipv4s: persist.discovery_bind_ipv4s,
            host_collect_gen: Arc::new(AtomicU64::new(0)),
            host_collect_active_sig: None,
            host_collect_broadcast: persist.host_collect_broadcast,
            host_collect_interval_secs: persist.host_collect_interval_secs.max(1),
            host_collect_poll_udp_port: persist.host_collect_poll_udp_port,
            host_collect_register_udp_port: persist.host_collect_register_udp_port,
            discovery_if_rows: Vec::new(),
            discovery_if_scan_secs: -1.0e6_f64,
            spoof_target_vm: String::new(),
            spoof_dynamic_mac: true,
            spoof_disable_checkpoints: false,
            pending_spoof_confirm_apply: false,
            ui_lang,
            settings_open: false,
            add_host_dialog_open: false,
            add_host_dialog_ip: String::new(),
            add_host_dialog_port: "7788".into(),
            add_host_dialog_err: String::new(),
            add_host_verify_busy: false,
            add_host_verify_session: 0,
            add_host_verify_deadline: None,
            ui_toast_until: None,
            ui_toast_text: String::new(),
            active_nav,
            really_quitting: false,
            hidden_to_tray: false,
            _tray: None,
            tray_icon_init_attempted: false,
            device_remark_edit_index: None,
            device_remark_edit_focus_next: false,
            device_masonry_heights: HashMap::new(),
            fleet_by_endpoint: HashMap::new(),
            fleet_busy: false,
            vm_inventory: Vec::new(),
            bulk_vm_names: String::new(),
            pending_confirm_stop: false,
            pending_confirm_start: false,
            last_action: String::new(),
            control_addr,
            net_tx,
            net_rx,
            net_busy: false,
            host_connected: false,
            command_ready: false,
            telemetry_live: false,
            last_host_telemetry_at: None,
            reachability_wall_anchor: Instant::now(),
            telemetry_links: HashMap::new(),
            host_disk_volumes: Vec::new(),
            last_capabilities: String::new(),
            last_net_error: String::new(),
        }
    }

    fn selected_endpoint_key(&self) -> Option<String> {
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
    pub(super) fn stop_dual_channels(&mut self) {
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

    fn mark_control_endpoint_offline(&mut self) {
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
    fn should_skip_probe_offline_for_addr(&self, addr_key: &str) -> bool {
        Self::endpoint_addr_key(&self.control_addr) == addr_key && self.host_connected
    }

    /// When `control_addr` is set and there is no session, periodically sends `Hello` (no manual Connect).
    fn tick_auto_control_session(&mut self, ctx: &egui::Context) {
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
    fn endpoint_addr_key(addr: &str) -> String {
        addr.trim().to_string()
    }

    fn stop_telemetry_reader_for_key(&mut self, host_key: &str) {
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

    fn remap_host_caches_addr_key(&mut self, old_key: &str, new_key: &str) {
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
    fn merge_add_host_after_verify(&mut self, addr: String, device_id: String, caps_summary: String) {
        let new_key = Self::endpoint_addr_key(&addr);
        if let Some(pos) = self.endpoints.iter().position(|e| e.device_id == device_id) {
            let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
            if old_key != new_key {
                self.stop_telemetry_reader_for_key(&old_key);
                self.remap_host_caches_addr_key(&old_key, &new_key);
                if old_key == Self::endpoint_addr_key(&self.control_addr) {
                    self.control_addr = addr.clone();
                    self.command_ready = false;
                    self.host_connected = false;
                    self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
                }
            }
            let ep = &mut self.endpoints[pos];
            ep.addr = addr;
            ep.last_caps = caps_summary;
            ep.last_known_online = true;
            return;
        }
        if let Some(pos) = self.endpoints.iter().position(|e| {
            Self::endpoint_addr_key(&e.addr) == new_key
                && (e.device_id.trim().is_empty()
                    || e.device_id == HostEndpoint::legacy_device_id_for_addr(&e.addr))
        }) {
            let ep = &mut self.endpoints[pos];
            ep.device_id = device_id;
            ep.addr = addr;
            ep.last_caps = caps_summary;
            ep.last_known_online = true;
            return;
        }
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

    /// Invalidate an in-flight add-host Hello (cancel, dialog close, or UI watchdog).
    pub(super) fn invalidate_add_host_probe(&mut self) {
        self.add_host_verify_session = self.add_host_verify_session.wrapping_add(1);
        self.add_host_verify_busy = false;
        self.add_host_verify_deadline = None;
    }

    fn tick_add_host_verify_watchdog(&mut self, ctx: &egui::Context) {
        if !self.add_host_verify_busy {
            return;
        }
        let Some(dl) = self.add_host_verify_deadline else {
            return;
        };
        let now = Instant::now();
        if now < dl {
            let wait = dl.saturating_duration_since(now).min(Duration::from_millis(400));
            ctx.request_repaint_after(wait);
            return;
        }
        self.invalidate_add_host_probe();
        self.ui_toast_text = i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
        self.ui_toast_until = Some(ctx.input(|i| i.time) + 3.8);
        ctx.request_repaint();
    }

    fn render_ui_toast(&self, ctx: &egui::Context) {
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
    fn upsert_endpoint_after_caps(&mut self, summary: String) {
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

    fn endpoint_mut_for_control_addr(&mut self) -> Option<&mut HostEndpoint> {
        let key = Self::endpoint_addr_key(&self.control_addr);
        if key.is_empty() {
            return None;
        }
        self.endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == key)
    }

    fn persist_snapshot(&self) -> CenterPersist {
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

    fn drain_net_inbox(&mut self) {
        while let Ok(msg) = self.net_rx.try_recv() {
            match msg {
                NetUiMsg::Caps { summary } => {
                    self.net_busy = false;
                    self.upsert_endpoint_after_caps(summary);
                    self.command_ready = true;
                    self.last_net_error.clear();
                    self.last_action = i18n::log_host_responded(self.ui_lang);
                    self.spawn_telemetry_reader();
                    self.recompute_host_connected();
                    self.ctx.request_repaint();
                }
                NetUiMsg::VmInventory(vms) => {
                    self.net_busy = false;
                    let n = vms.len();
                    let key = Self::endpoint_addr_key(&self.control_addr);
                    if !key.is_empty() {
                        let st = self.fleet_by_endpoint.entry(key).or_default();
                        st.vms = vms.clone();
                    }
                    self.vm_inventory = vms;
                    if let Some(ep) = self.endpoint_mut_for_control_addr() {
                        ep.last_vm_count = n as u32;
                    }
                    self.last_net_error.clear();
                    self.last_action = i18n::log_list_vms(self.ui_lang, n);
                }
                NetUiMsg::BatchStop {
                    succeeded,
                    failures,
                } => {
                    self.net_busy = false;
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_stop_vm_group(self.ui_lang, succeeded, failures.len());
                    if !failures.is_empty() {
                        self.last_net_error = failures.join("; ");
                    }
                }
                NetUiMsg::BatchStart {
                    succeeded,
                    failures,
                } => {
                    self.net_busy = false;
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_start_vm_group(self.ui_lang, succeeded, failures.len());
                    if !failures.is_empty() {
                        self.last_net_error = failures.join("; ");
                    }
                }
                NetUiMsg::SpoofApply {
                    dry_run,
                    steps,
                    notes,
                } => {
                    self.net_busy = false;
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_spoof_apply(self.ui_lang, dry_run, &steps.join(", "), &notes);
                }
                NetUiMsg::AddHostVerifyDone {
                    session_id,
                    addr,
                    ok,
                    device_id,
                    caps_summary,
                    error,
                } => {
                    if session_id != self.add_host_verify_session {
                        continue;
                    }
                    self.add_host_verify_busy = false;
                    self.add_host_verify_deadline = None;
                    if ok {
                        self.merge_add_host_after_verify(addr, device_id, caps_summary);
                        self.add_host_dialog_open = false;
                        self.add_host_dialog_err.clear();
                        self.persist_registered_devices();
                        self.last_net_error.clear();
                        self.last_action = i18n::t(self.ui_lang, Msg::AddHostSavedLog).to_string();
                    } else {
                        tracing::debug!(%addr, %error, "add host: Hello verify failed");
                        self.ui_toast_text =
                            i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
                        self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 3.8);
                    }
                    self.ctx.request_repaint();
                }
                NetUiMsg::HostAnnounced {
                    control_addr,
                    label,
                    device_id,
                } => {
                    let addr = Self::endpoint_addr_key(&control_addr);
                    if addr.is_empty() {
                        continue;
                    }
                    let id_from_host = device_id.trim();
                    let resolved_label = if label.trim().is_empty() {
                        format!("host-{}", addr.replace([':', '.'], "-"))
                    } else {
                        label.trim().to_string()
                    };
                    let new_addr = control_addr.trim().to_string();
                    let new_key = Self::endpoint_addr_key(&new_addr);

                    let lone_legacy_index = {
                        let hits: Vec<usize> = self
                            .endpoints
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| {
                                e.device_id.trim().is_empty()
                                    || e.device_id
                                        == HostEndpoint::legacy_device_id_for_addr(&e.addr)
                            })
                            .map(|(i, _)| i)
                            .collect();
                        if hits.len() == 1 {
                            Some(hits[0])
                        } else {
                            None
                        }
                    };

                    if !id_from_host.is_empty() {
                        if let Some(pos) = self
                            .endpoints
                            .iter()
                            .position(|e| e.device_id == id_from_host)
                        {
                            let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
                            if old_key != new_key {
                                self.stop_telemetry_reader_for_key(&old_key);
                                self.remap_host_caches_addr_key(&old_key, &new_key);
                                if old_key == Self::endpoint_addr_key(&self.control_addr) {
                                    self.control_addr = new_addr.clone();
                                    self.command_ready = false;
                                    self.host_connected = false;
                                    self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
                                }
                            }
                            let ep = &mut self.endpoints[pos];
                            ep.addr = new_addr;
                            if ep.label != resolved_label {
                                ep.label = resolved_label.clone();
                            }
                        } else if let Some(pos) = lone_legacy_index {
                            let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
                            if old_key != new_key {
                                self.stop_telemetry_reader_for_key(&old_key);
                                self.remap_host_caches_addr_key(&old_key, &new_key);
                                if old_key == Self::endpoint_addr_key(&self.control_addr) {
                                    self.control_addr = new_addr.clone();
                                    self.command_ready = false;
                                    self.host_connected = false;
                                    self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
                                }
                            }
                            let ep = &mut self.endpoints[pos];
                            ep.addr = new_addr.clone();
                            ep.device_id = id_from_host.to_string();
                            if ep.label != resolved_label {
                                ep.label = resolved_label.clone();
                            }
                        } else {
                            self.endpoints.push(HostEndpoint {
                                label: resolved_label.clone(),
                                addr: addr.clone(),
                                device_id: id_from_host.to_string(),
                                remark: String::new(),
                                last_caps: String::new(),
                                last_vm_count: 0,
                                last_known_online: false,
                            });
                        }
                    } else if let Some(ep) = self
                        .endpoints
                        .iter_mut()
                        .find(|e| Self::endpoint_addr_key(&e.addr) == addr)
                    {
                        if ep.label != resolved_label {
                            ep.label = resolved_label.clone();
                        }
                    } else {
                        self.endpoints.push(HostEndpoint {
                            label: resolved_label.clone(),
                            addr: addr.clone(),
                            device_id: HostEndpoint::legacy_device_id_for_addr(&addr),
                            remark: String::new(),
                            last_caps: String::new(),
                            last_vm_count: 0,
                            last_known_online: false,
                        });
                    }

                    self.persist_registered_devices();
                    self.last_net_error.clear();
                    self.last_action = i18n::log_lan_host_announced(
                        self.ui_lang,
                        &resolved_label,
                        &addr,
                    );
                    self.ctx.request_repaint();
                }
                NetUiMsg::HostResources {
                    control_addr,
                    stats,
                } => {
                    self.host_resource_stats.insert(control_addr, stats);
                    self.ctx.request_repaint();
                }
                NetUiMsg::DesktopSnapshot {
                    control_addr,
                    jpeg_bytes,
                } => match image::load_from_memory(&jpeg_bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image =
                            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                        let tex = self.ctx.load_texture(
                            format!("host_desktop_{control_addr}"),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.host_desktop_textures.insert(control_addr, tex);
                        self.ctx.request_repaint();
                    }
                    Err(e) => {
                        tracing::warn!(%control_addr, %e, len = jpeg_bytes.len(), "desktop preview: JPEG decode failed");
                    }
                },
                NetUiMsg::DesktopFetchCycleDone => {
                    self.desktop_fetch_busy = false;
                }
                NetUiMsg::HostReachability {
                    control_addr,
                    online,
                } => {
                    let key = Self::endpoint_addr_key(&control_addr);
                    let skip_offline = !online && self.should_skip_probe_offline_for_addr(&key);
                    if let Some(ep) = self
                        .endpoints
                        .iter_mut()
                        .find(|e| Self::endpoint_addr_key(&e.addr) == key)
                    {
                        if online {
                            ep.last_known_online = true;
                        } else if !skip_offline {
                            ep.last_known_online = false;
                        }
                    }
                    self.ctx.request_repaint();
                }
                NetUiMsg::ReachabilityProbeCycleDone => {
                    self.reachability_probe_busy = false;
                }
                NetUiMsg::HostTelemetry {
                    host_key,
                    gen,
                    push,
                } => {
                    if self
                        .telemetry_links
                        .get(&host_key)
                        .is_none_or(|l| l.session_gen != gen)
                    {
                        continue;
                    }
                    let st = self.fleet_by_endpoint.entry(host_key.clone()).or_default();
                    match push {
                        titan_common::ControlPush::HostTelemetry {
                            vms,
                            volumes,
                            content_hint,
                        } => {
                            let n = vms.len();
                            st.vms = vms.clone();
                            st.volumes = volumes.clone();
                            st.telemetry_live = true;
                            st.last_telemetry_at = Some(Instant::now());
                            if self.selected_endpoint_key().as_ref() == Some(&host_key) {
                                self.vm_inventory = vms.clone();
                                self.host_disk_volumes = volumes.clone();
                                if let Some(ep) = self.endpoint_mut_for_control_addr() {
                                    ep.last_vm_count = n as u32;
                                    ep.last_known_online = true;
                                }
                            }
                            if let Some(ep) = self
                                .endpoints
                                .iter_mut()
                                .find(|e| Self::endpoint_addr_key(&e.addr) == host_key)
                            {
                                ep.last_vm_count = n as u32;
                                ep.last_known_online = true;
                            }
                            if let Some(h) = content_hint {
                                if !h.is_empty() {
                                    self.last_action = h;
                                }
                            }
                        }
                        titan_common::ControlPush::HostResourceLive { stats } => {
                            if !host_key.is_empty() {
                                self.host_resource_stats.insert(host_key.clone(), stats);
                            }
                        }
                        titan_common::ControlPush::HostDesktopPreviewJpeg {
                            jpeg_bytes, ..
                        } => {
                            if !host_key.is_empty() {
                                match image::load_from_memory(&jpeg_bytes) {
                                    Ok(img) => {
                                        let rgba = img.to_rgba8();
                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                            size,
                                            rgba.as_raw(),
                                        );
                                        let tex = self.ctx.load_texture(
                                            format!("host_desktop_{host_key}"),
                                            color_image,
                                            egui::TextureOptions::LINEAR,
                                        );
                                        self.host_desktop_textures.insert(host_key.clone(), tex);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            %host_key,
                                            %e,
                                            len = jpeg_bytes.len(),
                                            "telemetry desktop preview: JPEG decode failed"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    if host_key == Self::endpoint_addr_key(&self.control_addr) {
                        self.telemetry_live = true;
                        self.last_host_telemetry_at = Some(Instant::now());
                    }
                    self.last_net_error.clear();
                    self.recompute_host_connected();
                    self.ctx.request_repaint();
                }
                NetUiMsg::TelemetryLinkLost { host_key, gen } => {
                    if self
                        .telemetry_links
                        .get(&host_key)
                        .is_none_or(|l| l.session_gen != gen)
                    {
                        continue;
                    }
                    if let Some(s) = self.fleet_by_endpoint.get_mut(&host_key) {
                        s.clear_telemetry();
                    }
                    self.host_resource_stats.remove(&host_key);
                    self.host_desktop_textures.remove(&host_key);
                    if host_key == Self::endpoint_addr_key(&self.control_addr) {
                        self.telemetry_live = false;
                        self.last_host_telemetry_at = None;
                        self.recompute_host_connected();
                        self.mark_control_endpoint_offline();
                    }
                    self.ctx.request_repaint();
                }
                NetUiMsg::FleetOpResult {
                    host_key,
                    ok,
                    detail,
                } => {
                    if !host_key.is_empty() {
                        self.last_action = if ok {
                            format!("{host_key}: OK")
                        } else {
                            format!("{host_key}: {detail}")
                        };
                        if !ok {
                            if !self.last_net_error.is_empty() {
                                self.last_net_error.push_str("; ");
                            }
                            self.last_net_error
                                .push_str(&format!("{host_key}: {detail}"));
                        }
                    }
                    self.ctx.request_repaint();
                }
                NetUiMsg::FleetOpDone => {
                    self.fleet_busy = false;
                    self.ctx.request_repaint();
                }
                NetUiMsg::Error(e) => {
                    self.net_busy = false;
                    self.last_net_error = e;
                    self.last_action = i18n::log_request_failed(self.ui_lang);
                    // Hello / one-shot RPC failures while not on a live telemetry stream: reflect offline on the card.
                    if !self.telemetry_live {
                        self.mark_control_endpoint_offline();
                    }
                    self.ctx.request_repaint();
                }
            }
        }
    }

    fn prune_host_desktop_textures(&mut self) {
        let valid: std::collections::HashSet<_> = self
            .endpoints
            .iter()
            .map(|e| Self::endpoint_addr_key(&e.addr))
            .collect();
        self.host_desktop_textures.retain(|k, _| valid.contains(k));
        self.host_resource_stats.retain(|k, _| valid.contains(k));
        self.fleet_by_endpoint.retain(|k, _| valid.contains(k));
    }

    fn tick_desktop_preview_refresh(&mut self, ctx: &egui::Context) {
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

    pub(super) fn refresh_discovery_iface_rows(&mut self, ui: &egui::Ui) {
        let now = ui.ctx().input(|i| i.time);
        let initial_scan = self.discovery_if_scan_secs < -100_000.0;
        if initial_scan || now - self.discovery_if_scan_secs >= 3.0 {
            self.discovery_if_scan_secs = now;
            self.discovery_if_rows = discovery::list_lan_ipv4_rows();
        }
    }

    fn tick_discovery_thread(&mut self) {
        let want = self.discovery_broadcast;
        let sig = discovery::DiscoverySpawnSig::new(
            self.discovery_interval_secs.max(1),
            self.discovery_udp_port,
            self.control_addr.clone(),
            self.discovery_bind_ipv4s.clone(),
        );

        if want {
            let need_spawn = self.discovery_active_sig.as_ref() != Some(&sig);
            if need_spawn {
                if self.discovery_active_sig.is_some() {
                    self.discovery_gen.fetch_add(1, Ordering::SeqCst);
                }
                let my_gen = self.discovery_gen.fetch_add(1, Ordering::SeqCst) + 1;
                let gen = self.discovery_gen.clone();
                let interval = Duration::from_secs(u64::from(sig.interval_secs));
                let port = sig.port;
                let host_control = sig.host_control.clone();
                let bind = sig.bind_ipv4s.clone();
                std::thread::spawn(move || {
                    discovery::discovery_udp_loop(my_gen, gen, interval, port, host_control, bind);
                });
                self.discovery_active_sig = Some(sig);
            }
        } else if self.discovery_active_sig.is_some() {
            self.discovery_gen.fetch_add(1, Ordering::SeqCst);
            self.discovery_active_sig = None;
        }
    }

    fn tick_host_collect_thread(&mut self) {
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
                if self.host_collect_active_sig.is_some() {
                    self.host_collect_gen.fetch_add(1, Ordering::SeqCst);
                }
                let my_gen = self.host_collect_gen.fetch_add(1, Ordering::SeqCst) + 1;
                let gen = self.host_collect_gen.clone();
                let interval = Duration::from_secs(u64::from(sig.interval_secs));
                let poll_port = sig.poll_port;
                let register_port = sig.register_port;
                let bind = sig.bind_ipv4s.clone();
                std::thread::spawn(move || {
                    discovery::center_host_collect_udp_loop(
                        my_gen,
                        gen,
                        interval,
                        poll_port,
                        register_port,
                        bind,
                    );
                });
                self.host_collect_active_sig = Some(sig);
            }
        } else if self.host_collect_active_sig.is_some() {
            self.host_collect_gen.fetch_add(1, Ordering::SeqCst);
            self.host_collect_active_sig = None;
        }
    }

    fn tick_list_vms_auto_refresh(&mut self, ctx: &egui::Context) {
        if !self.host_connected {
            self.list_vms_poll_accum = 0.0;
            return;
        }
        // Event-driven telemetry is the online inventory source; do not poll ListVms on a timer.
        if self.telemetry_live {
            self.list_vms_poll_accum = 0.0;
            return;
        }
        if self.list_vms_auto_refresh
            && !self.net_busy
            && !self.fleet_busy
            && !self.control_addr.trim().is_empty()
        {
            self.list_vms_poll_accum += ctx.input(|i| i.unstable_dt);
            if self.list_vms_poll_accum >= self.list_vms_poll_secs.max(5) as f32 {
                self.list_vms_poll_accum = 0.0;
                self.spawn_list_vms();
            }
        }
    }

    fn recompute_host_connected(&mut self) {
        self.host_connected = self.command_ready && self.telemetry_live;
    }

    fn tick_reachability_probes(&mut self, _ctx: &egui::Context) {
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
            self.spawn_reachability_probe_cycle();
        }
    }

    fn tick_telemetry_staleness(&mut self) {
        if !self.telemetry_live {
            return;
        }
        let Some(t) = self.last_host_telemetry_at else {
            return;
        };
        if t.elapsed() <= Duration::from_secs_f64(TELEMETRY_STALE_AFTER_SECS) {
            return;
        }
        self.telemetry_live = false;
        self.last_host_telemetry_at = None;
        self.recompute_host_connected();
        self.ctx.request_repaint();
    }

    /// Changing the control address invalidates both TCP planes (command + paired telemetry port).
    pub(super) fn on_control_addr_changed(&mut self) {
        self.stop_dual_channels();
        self.vm_inventory.clear();
        self.host_disk_volumes.clear();
        self.fleet_by_endpoint.clear();
        self.list_vms_poll_accum = 0.0;
        self.last_capabilities.clear();
        self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
    }

    pub(super) fn select_endpoint_host(&mut self, index: usize) {
        if index >= self.endpoints.len() {
            return;
        }
        let new = self.endpoints[index].addr.clone();
        if Self::endpoint_addr_key(&new) != Self::endpoint_addr_key(&self.control_addr) {
            self.on_control_addr_changed();
        }
        self.selected_host = index;
        self.control_addr = new;
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        let visuals = ctx.style().visuals.clone();
        let top_fill = visuals.panel_fill;
        // Bottom edge: `show_separator_line` only (matches stroke weight with side nav rule).
        egui::TopBottomPanel::top("status")
            .frame(
                Frame::NONE
                    .fill(top_fill)
                    .inner_margin(Margin::symmetric(22, 12))
                    .stroke(Stroke::NONE)
                    .shadow(card_shadow()),
            )
            .show_separator_line(true)
            .default_height(52.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    self.top_status_bar(ui);
                });
            });
    }

    fn render_side_nav(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let visuals = ctx.style().visuals.clone();
        let nav_fill = visuals.extreme_bg_color;
        egui::SidePanel::left("nav")
            .exact_width(SIDEBAR_DEFAULT_WIDTH)
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(nav_fill)
                    .inner_margin(Margin::symmetric(10, 14))
                    // Right edge is the panel separator only (light gray in theme).
                    .stroke(Stroke::NONE),
            )
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                let tabs = [
                    (NavTab::Monitor, Msg::NavMonitor),
                    (NavTab::Connect, Msg::NavConnect),
                    (NavTab::HostsVms, Msg::NavHostsVms),
                    (NavTab::Spoof, Msg::NavSpoof),
                    (NavTab::Power, Msg::NavPower),
                    (NavTab::Settings, Msg::NavSettings),
                ];
                let mut nav_row = |ui: &mut egui::Ui, tab: NavTab, msg: Msg| {
                    let selected = self.active_nav == tab;
                    let w = ui.available_width();
                    let label = i18n::t(lang, msg);
                    let inactive = ui.visuals().widgets.inactive.text_color();
                    let text = if selected {
                        RichText::new(label).size(14.0).strong().color(ACCENT)
                    } else {
                        RichText::new(label).size(14.0).color(inactive)
                    };
                    let galley = WidgetText::from(text).into_galley(
                        ui,
                        Some(TextWrapMode::Extend),
                        f32::INFINITY,
                        TextStyle::Button,
                    );
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(w, NAV_ITEM_HEIGHT), Sense::click());
                    if ui.is_rect_visible(rect) {
                        let y = rect.center().y - 0.5 * galley.size().y;
                        ui.painter()
                            .galley(egui::pos2(rect.min.x, y), galley, inactive);
                    }
                    if response.clicked() {
                        self.active_nav = tab;
                    }
                };
                for &(tab, msg) in &tabs {
                    nav_row(ui, tab, msg);
                }
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        let central_fill = ctx.style().visuals.window_fill;
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(central_fill)
                    .inner_margin(Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(self.active_nav as u8)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let full_w = ui.available_width();
                        let content_w = effective_content_width(full_w);
                        let column_w = content_w.min(full_w);
                        let side_remain = (full_w - column_w).max(0.0);
                        let (left_pad, right_pad) = if self.active_nav == NavTab::Settings {
                            (0.0, side_remain)
                        } else {
                            let half = 0.5 * side_remain;
                            (half, half)
                        };
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            ui.vertical(|ui| {
                                ui.set_width(column_w);
                                match self.active_nav {
                                    NavTab::Connect => self.panel_device_management_redirect(ui),
                                    NavTab::Settings => self.panel_settings_host(ui),
                                    NavTab::HostsVms => self.panel_window_management(ui),
                                    NavTab::Monitor => self.panel_resource_monitor(ui),
                                    NavTab::Spoof => self.panel_spoof_pipeline(ui, ctx),
                                    NavTab::Power => self.panel_danger(ui, ctx),
                                }
                            });
                            ui.add_space(right_pad);
                        });
                    });
            });
    }

    fn render_settings_window(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let mut close_clicked = false;
        egui::Window::new(i18n::t(lang, Msg::SettingsTitle))
            .open(&mut self.settings_open)
            .collapsible(false)
            .resizable(false)
            .default_pos(ctx.screen_rect().right_top() + egui::vec2(-256.0, 48.0))
            .default_width(248.0)
            .show(ctx, |ui| {
                ui.radio_value(
                    &mut self.ui_lang,
                    UiLang::En,
                    i18n::t(lang, Msg::LangRadioEn),
                );
                ui.radio_value(
                    &mut self.ui_lang,
                    UiLang::Zh,
                    i18n::t(lang, Msg::LangRadioZh),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new(i18n::t(lang, Msg::SettingsMoreLangNote))
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(8.0);
                if ui.button(i18n::t(lang, Msg::SettingsClose)).clicked() {
                    close_clicked = true;
                }
            });
        if close_clicked {
            self.settings_open = false;
        }
    }

    /// Persist the registered device list to SQLite (same store as app shutdown).
    pub(super) fn persist_registered_devices(&self) {
        let db_path = device_store::registration_db_path();
        if let Err(e) = device_store::save_registered_devices(&db_path, &self.endpoints) {
            tracing::warn!("device_store: save {:?}: {e}", db_path);
        }
    }
}

impl eframe::App for CenterApp {
    /// Intercept native close **before** egui consumes it: `eframe` snapshots `close_requested`
    /// prior to this hook, then requires [`egui::ViewportCommand::CancelClose`] in the same frame’s
    /// output — handling only inside [`Self::update`] is unreliable for “hide to tray”.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.really_quitting || raw_input.viewport_id != egui::ViewportId::ROOT {
            return;
        }
        if !raw_input.viewport().close_requested() {
            return;
        }
        if let Some(vp) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            vp.events.retain(|e| *e != egui::ViewportEvent::Close);
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Visible(false),
        );
        ctx.request_repaint_after_for(
            std::time::Duration::from_millis(250),
            egui::ViewportId::ROOT,
        );
        self.hidden_to_tray = true;
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persist_registered_devices();
        if let Ok(json) = serde_json::to_string(&self.persist_snapshot()) {
            storage.set_string(PERSIST_KEY, json);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.maybe_init_tray_icon_once();
        if let Some(until) = self.ui_toast_until {
            if ctx.input(|i| i.time) >= until {
                self.ui_toast_until = None;
                self.ui_toast_text.clear();
            }
        }
        if titan_tray::poll_tray_for_egui(ctx, &mut self.really_quitting) {
            self.hidden_to_tray = false;
        }
        if self.hidden_to_tray {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }
        self.drain_net_inbox();
        self.tick_add_host_verify_watchdog(ctx);
        self.tick_telemetry_staleness();
        self.tick_reachability_probes(ctx);
        self.tick_auto_control_session(ctx);
        self.tick_discovery_thread();
        self.tick_host_collect_thread();
        self.tick_desktop_preview_refresh(ctx);
        self.tick_list_vms_auto_refresh(ctx);
        self.render_top_panel(ctx);
        self.render_side_nav(ctx);
        self.render_central_panel(ctx);
        self.render_settings_window(ctx);
        self.render_ui_toast(ctx);
    }
}
