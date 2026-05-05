//! `CenterApp` construction and tray bootstrap.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;
use std::time::Instant;

use crate::app::CenterApp;
use crate::app::CenterSecurity;
use crate::app::constants::{DESKTOP_PREVIEW_POLL_SECS, REACHABILITY_PROBE_SECS};
use crate::app::device_store;
use crate::app::i18n::UiLang;
use crate::app::lan_host_register;
use crate::app::persist_data::{
    CenterPersist, NavTab, default_discovery_interval_secs, default_discovery_udp_port,
    default_host_collect_interval_secs, default_list_vms_poll_secs,
};
use crate::app::ui::theme::apply_center_theme;

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

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_center_theme(&cc.egui_ctx);
        let (net_tx, net_rx) = mpsc::channel();
        let db_path = device_store::registration_db_path();
        let load = load_center_persist(&db_path);
        lan_host_register::spawn_center_lan_host_register_listener(
            net_tx.clone(),
            cc.egui_ctx.clone(),
            load.persist.host_collect_register_udp_port.max(1),
        );
        let endpoints = load_or_migrate_endpoints(&db_path, load.legacy_eps);
        let control_addr = endpoints
            .first()
            .map(|e| e.addr.clone())
            .unwrap_or_default();
        let center_security = init_center_security();
        let app = Self::assemble_center_app(
            cc,
            net_tx,
            net_rx,
            load.persist,
            endpoints,
            control_addr,
            center_security,
        );
        app.flush_center_settings_to_sqlite();
        tracing::info!(db = %db_path.display(), "center sqlite snapshot after startup");
        app
    }

    fn assemble_center_app(
        cc: &eframe::CreationContext<'_>,
        net_tx: mpsc::Sender<crate::app::net::NetUiMsg>,
        net_rx: mpsc::Receiver<crate::app::net::NetUiMsg>,
        persist: CenterPersist,
        endpoints: Vec<crate::app::HostEndpoint>,
        control_addr: String,
        center_security: CenterSecurity,
    ) -> Self {
        let active_nav = persist.active_nav.normalize();
        let ui_lang = persist.ui_lang;
        let desktop_poll_accum = derive_desktop_poll_accum(active_nav);
        let auto_hello_accum = derive_auto_hello_accum(&control_addr);
        Self::build_center_app_struct(BuildArgs {
            ctx: cc.egui_ctx.clone(),
            persist,
            endpoints,
            control_addr,
            center_security,
            net_tx,
            net_rx,
            ui_lang,
            active_nav,
            desktop_poll_accum,
            auto_hello_accum,
        })
    }

    fn build_center_app_struct(args: BuildArgs) -> Self {
        let v = build_initial_view();
        let dyn_fields = DynamicFields::from_args(&args);
        let BuildArgs {
            ctx,
            persist,
            endpoints,
            center_security,
            ui_lang,
            active_nav,
            net_tx,
            net_rx,
            control_addr,
            ..
        } = args;
        let net = build_initial_net(net_tx, net_rx, control_addr);
        Self::initial_state(StateInit {
            ctx,
            persist,
            endpoints,
            center_security,
            ui_lang,
            active_nav,
            v,
            net,
            d: dyn_fields,
        })
    }

    #[rustfmt::skip]
    fn initial_state(s: StateInit) -> Self {
        // Pure assembly of the CenterApp god-struct. Splitting CenterApp into sub-structs is a
        // follow-up refactor (cross-cutting `self.foo` rename); kept compact here so this fn
        // stays under the 30-line limit without that broader churn.
        let StateInit { ctx, persist, endpoints, center_security, ui_lang, active_nav, v, net, d } = s;
        Self {
            center_security, tofu_pending: None, ctx, endpoints, selected_host: 0, accounts: persist.accounts, proxy_labels: persist.proxy_labels, last_script_version: persist.last_script_version, list_vms_auto_refresh: persist.list_vms_auto_refresh, list_vms_poll_secs: persist.list_vms_poll_secs.max(5),
            list_vms_poll_accum: 0.0, auto_hello_accum: d.auto_hello_accum, desktop_poll_accum: d.desktop_poll_accum, prev_nav_for_desktop: active_nav, desktop_fetch_busy: false, reachability_poll_accum: REACHABILITY_PROBE_SECS, reachability_probe_busy: false,
            host_desktop_textures: HashMap::new(), host_resource_stats: HashMap::new(), discovery_gen: Arc::new(AtomicU64::new(0)), discovery_active_sig: None, discovery_broadcast: persist.discovery_broadcast, discovery_interval_secs: persist.discovery_interval_secs.max(1),
            discovery_udp_port: persist.discovery_udp_port, discovery_bind_ipv4s: persist.discovery_bind_ipv4s, host_collect_gen: Arc::new(AtomicU64::new(0)), host_collect_active_sig: None, host_collect_broadcast: persist.host_collect_broadcast, host_collect_interval_secs: persist.host_collect_interval_secs.max(1),
            host_collect_poll_udp_port: persist.host_collect_poll_udp_port, host_collect_register_udp_port: persist.host_collect_register_udp_port, discovery_if_rows: Vec::new(), discovery_if_scan_secs: -1.0e6_f64, ui_lang, host_synced_ui_lang: ui_lang, settings_open: false, settings_lang_btn_rect: None,
            add_host_dialog_open: false, add_host_dialog_ip: String::new(), add_host_dialog_port: "7788".into(), add_host_dialog_err: String::new(), add_host_verify_busy: false, add_host_verify_session: 0, add_host_verify_deadline: None, ui_toast_until: None, ui_toast_text: String::new(), active_nav,
            really_quitting: false, hidden_to_tray: false, _tray: None, tray_icon_init_attempted: false, device_remark_edit_index: None, device_remark_edit_focus_next: false, device_masonry_heights: HashMap::new(), vm_window_masonry_heights: HashMap::new(),
            vm_window_create: crate::app::vm_window_create_dialog::CenterVmWindowCreateForm::with_defaults(), vm_window_create_id_nonce: 0, pending_remove_endpoint: None, pending_delete_vm_window_row_ix: None, host_config_window_open: false, host_managed_draft_json: String::new(), host_managed_last_msg: String::new(),
            fleet_by_endpoint: HashMap::new(), fleet_busy: false, vm_inventory: Vec::new(), vm_window_records: v.vm_window_records, last_action: String::new(), control_addr: net.control_addr, net_tx: net.net_tx, net_rx: net.net_rx,
            net_busy: false, host_connected: false, command_ready: false, telemetry_live: false, last_host_telemetry_at: None, reachability_wall_anchor: Instant::now(), telemetry_links: HashMap::new(), host_disk_volumes: Vec::new(),
            last_capabilities: String::new(), last_net_error: String::new(), sqlite_snapshot_last_time: -1.0e9_f64,
        }
    }
}

struct ViewDefaults {
    vm_window_records: Vec<titan_common::VmWindowRecord>,
}

struct NetDefaults {
    net_tx: mpsc::Sender<crate::app::net::NetUiMsg>,
    net_rx: mpsc::Receiver<crate::app::net::NetUiMsg>,
    control_addr: String,
}

struct DynamicFields {
    desktop_poll_accum: f32,
    auto_hello_accum: f32,
}

impl DynamicFields {
    fn from_args(args: &BuildArgs) -> Self {
        Self {
            desktop_poll_accum: args.desktop_poll_accum,
            auto_hello_accum: args.auto_hello_accum,
        }
    }
}

struct StateInit {
    ctx: egui::Context,
    persist: CenterPersist,
    endpoints: Vec<crate::app::HostEndpoint>,
    center_security: CenterSecurity,
    ui_lang: UiLang,
    active_nav: NavTab,
    v: ViewDefaults,
    net: NetDefaults,
    d: DynamicFields,
}

fn build_initial_view() -> ViewDefaults {
    ViewDefaults {
        vm_window_records: crate::app::vm_window_db::list_all(
            &crate::app::vm_window_db::center_vm_window_db_path(),
        )
        .unwrap_or_default(),
    }
}

fn build_initial_net(
    net_tx: mpsc::Sender<crate::app::net::NetUiMsg>,
    net_rx: mpsc::Receiver<crate::app::net::NetUiMsg>,
    control_addr: String,
) -> NetDefaults {
    NetDefaults {
        net_tx,
        net_rx,
        control_addr,
    }
}

fn derive_desktop_poll_accum(active_nav: NavTab) -> f32 {
    if active_nav == NavTab::Connect {
        DESKTOP_PREVIEW_POLL_SECS
    } else {
        0.0
    }
}

fn derive_auto_hello_accum(control_addr: &str) -> f32 {
    if control_addr.trim().is_empty() {
        0.0
    } else {
        CenterApp::AUTO_HELLO_RETRY_SECS
    }
}

struct BuildArgs {
    ctx: egui::Context,
    persist: CenterPersist,
    endpoints: Vec<crate::app::HostEndpoint>,
    control_addr: String,
    center_security: CenterSecurity,
    net_tx: mpsc::Sender<crate::app::net::NetUiMsg>,
    net_rx: mpsc::Receiver<crate::app::net::NetUiMsg>,
    ui_lang: UiLang,
    active_nav: NavTab,
    desktop_poll_accum: f32,
    auto_hello_accum: f32,
}

struct CenterPersistLoad {
    persist: CenterPersist,
    legacy_eps: Option<Vec<crate::app::HostEndpoint>>,
}

fn load_center_persist(db_path: &std::path::Path) -> CenterPersistLoad {
    let persist_json = device_store::load_center_persist_json(db_path)
        .ok()
        .flatten();
    let legacy_eps = persist_json
        .as_deref()
        .and_then(device_store::legacy_endpoints_from_center_json);
    let persist: CenterPersist = persist_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_else(default_center_persist);
    CenterPersistLoad {
        persist,
        legacy_eps,
    }
}

fn default_center_persist() -> CenterPersist {
    CenterPersist {
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
    }
}

fn load_or_migrate_endpoints(
    db_path: &std::path::Path,
    legacy_eps: Option<Vec<crate::app::HostEndpoint>>,
) -> Vec<crate::app::HostEndpoint> {
    let mut endpoints = device_store::load_registered_devices(db_path).unwrap_or_else(|e| {
        tracing::warn!("device_store: load {:?}: {e}", db_path);
        Vec::new()
    });
    if endpoints.is_empty()
        && let Some(import) = legacy_eps.filter(|e| !e.is_empty())
    {
        endpoints = import;
        if let Err(e) = device_store::save_registered_devices(db_path, &endpoints) {
            tracing::warn!("device_store: migrate save {:?}: {e}", db_path);
        }
    }
    for ep in &mut endpoints {
        ep.ensure_device_id();
    }
    endpoints
}

fn init_center_security() -> CenterSecurity {
    use titan_quic::{Role, TrustStore, load_or_generate};
    let identity_dir = crate::app::center_paths::identity_dir();
    let trust_path = crate::app::center_paths::trust_store_path();
    let device_id = device_id_for_center();
    let identity = match load_or_generate(&identity_dir, Role::Center, &device_id) {
        Ok(id) => Arc::new(id),
        Err(e) => panic!("titan-center: cannot load/generate mTLS identity: {e}"),
    };
    let trust = match TrustStore::open(trust_path) {
        Ok(t) => Arc::new(t),
        Err(e) => panic!("titan-center: cannot open trust store: {e}"),
    };
    titan_quic::install_default_crypto_provider();
    if let Err(e) = crate::app::net::init_global(identity.clone(), trust.clone()) {
        panic!("titan-center: cannot init QUIC client: {e}");
    }
    tracing::info!(
        device_id = %device_id,
        fingerprint = %identity.spki_sha256_hex,
        "center mTLS identity ready"
    );
    CenterSecurity { identity, trust }
}

fn device_id_for_center() -> String {
    machine_uid::get().unwrap_or_else(|_| "unknown-center".to_string())
}
