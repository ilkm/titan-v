//! `CenterApp` construction and tray bootstrap.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Instant;

use super::constants::{DESKTOP_PREVIEW_POLL_SECS, REACHABILITY_PROBE_SECS};
use super::device_store;
use super::i18n::UiLang;
use super::lan_host_register;
use super::persist_data::{
    default_discovery_interval_secs, default_discovery_udp_port,
    default_host_collect_interval_secs, default_list_vms_poll_secs, CenterPersist, NavTab,
};
use super::theme::apply_center_theme;
use super::CenterApp;

impl CenterApp {
    /// Interval between automatic `Hello` attempts when not connected (`control_addr` non-empty).
    pub(crate) const AUTO_HELLO_RETRY_SECS: f32 = 3.0;

    /// Build the status tray once the winit/eframe loop is running (required on macOS).
    pub(crate) fn maybe_init_tray_icon_once(&mut self) {
        if self.tray_icon_init_attempted {
            return;
        }
        self.tray_icon_init_attempted = true;

        match titan_tray::build_tray_icon(self.ui_lang) {
            Ok(t) => self._tray = Some(t),
            Err(e) => tracing::warn!("system tray unavailable: {e}"),
        }
    }

    pub(crate) fn sync_tray_glyph_lang(&mut self) {
        let Some(tray) = self._tray.as_ref() else {
            return;
        };
        if self.tray_glyph_lang == self.ui_lang {
            return;
        }
        if let Err(e) =
            titan_tray::refresh_tray_icon(tray, titan_tray::DesktopProduct::Center, self.ui_lang)
        {
            tracing::warn!("tray icon refresh: {e}");
            return;
        }
        self.tray_glyph_lang = self.ui_lang;
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_center_theme(&cc.egui_ctx);
        let (net_tx, net_rx) = mpsc::channel();
        let db_path = device_store::registration_db_path();
        let persist_json = device_store::load_center_persist_json(&db_path)
            .ok()
            .flatten();
        let legacy_eps = persist_json
            .as_deref()
            .and_then(device_store::legacy_endpoints_from_center_json);
        let persist: CenterPersist = persist_json
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
        let active_nav = persist.active_nav.normalize();
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
        let app = Self {
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
            ui_lang,
            host_synced_ui_lang: ui_lang,
            settings_open: false,
            settings_lang_btn_rect: None,
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
            tray_glyph_lang: ui_lang,
            device_remark_edit_index: None,
            device_remark_edit_focus_next: false,
            device_masonry_heights: HashMap::new(),
            pending_remove_endpoint: None,
            host_config_window_open: false,
            host_managed_draft_json: String::new(),
            host_managed_last_msg: String::new(),
            fleet_by_endpoint: HashMap::new(),
            fleet_busy: false,
            vm_inventory: Vec::new(),
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
            sqlite_snapshot_last_time: -1.0e9_f64,
        };
        app.flush_center_settings_to_sqlite();
        tracing::info!(db = %db_path.display(), "center sqlite snapshot after startup");
        app
    }
}
