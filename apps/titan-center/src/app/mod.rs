//! Center UI: persisted host table, control-plane (multi-message TCP), VM inventory, window masonry.

pub mod center_paths;
mod constants;
pub mod device_store;
mod discovery;
mod fleet_state;
mod vm_window_create_dialog;
pub mod vm_window_db;
pub mod vm_window_push_to_hosts;
pub use titan_i18n as i18n;
mod lan_host_register;
pub mod net;
mod persist_data;
mod spawn;
mod ui;

mod center_shell;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use titan_common::{HostResourceStats, VmWindowRecord};
use titan_quic::{Identity, TrustStore};

pub use persist_data::{HostEndpoint, NavTab};

use self::fleet_state::HostLiveState;
use self::i18n::UiLang;
use self::net::NetUiMsg;

/// Center-side mTLS identity + trust store, shared across UI and background workers.
pub struct CenterSecurity {
    pub identity: Arc<Identity>,
    pub trust: Arc<TrustStore>,
}

/// Pending TOFU (trust on first use) prompt for a manually-entered Host whose fingerprint
/// is not yet in the trust store.
#[derive(Debug, Clone)]
pub struct TofuPrompt {
    pub host_addr: String,
    pub fingerprint: String,
    pub label: String,
}

/// Center manager application state (UI thread).
/// One background telemetry TCP session for a given endpoint key ([`CenterApp::endpoint_addr_key`]).
pub(crate) struct TelemetryLink {
    /// Generation for [`NetUiMsg::HostTelemetry::session_gen`] / [`NetUiMsg::TelemetryLinkLost::session_gen`]; bumps on stop or new reader.
    pub(crate) session_gen: u64,
    pub(crate) stop: Arc<AtomicBool>,
    pub(crate) running: Arc<AtomicBool>,
}

pub struct CenterApp {
    pub(crate) center_security: CenterSecurity,
    /// Pending TOFU dialog data (`Some` when a manually-added host's fingerprint is unknown);
    /// drained by [`CenterApp::confirm_tofu_pending`] / [`CenterApp::dismiss_tofu_pending`].
    pub(crate) tofu_pending: Option<TofuPrompt>,
    pub(crate) ctx: egui::Context,
    pub(crate) endpoints: Vec<HostEndpoint>,
    pub(crate) selected_host: usize,
    pub(crate) accounts: Vec<String>,
    pub(crate) proxy_labels: Vec<String>,
    /// Per-endpoint inventory / telemetry (fleet console).
    pub(crate) fleet_by_endpoint: HashMap<String, HostLiveState>,
    pub(crate) fleet_busy: bool,
    pub(crate) vm_inventory: Vec<titan_common::VmBrief>,
    /// Host-reported planned VM windows (SQLite + LAN UDP).
    pub(crate) vm_window_records: Vec<VmWindowRecord>,
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
    pub(crate) ui_lang: UiLang,
    /// Last language pushed to registered hosts via [`ControlRequest::SetUiLang`]; resync when it diverges from [`Self::ui_lang`].
    pub(crate) host_synced_ui_lang: UiLang,
    pub(crate) settings_open: bool,
    /// Last frame's 🌐 button rect (screen space); used to anchor the settings popup.
    pub(crate) settings_lang_btn_rect: Option<egui::Rect>,
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
    /// Owns the tray icon when `tray-icon` successfully builds (platform-dependent).
    pub(crate) _tray: Option<titan_tray::TrayIcon>,
    /// macOS/Winit: tray must be created after the event loop has started (`StartCause::Init`); see tray-icon docs.
    tray_icon_init_attempted: bool,
    /// Device card: index into `endpoints` whose remark is being edited (`None` = display mode).
    pub(crate) device_remark_edit_index: Option<usize>,
    /// Request focus on the remark `TextEdit` the first frame after opening edit mode.
    device_remark_edit_focus_next: bool,
    /// Window-mgmt card: `record_id` of the row whose remark is being edited (`None` = display).
    /// Stored by record id (not row index) so masonry reorder / snapshot replace can't race the edit.
    pub(crate) vm_window_remark_edit_record_id: Option<String>,
    /// Request focus on the VM-window remark `TextEdit` the first frame after entering edit mode.
    pub(crate) vm_window_remark_edit_focus_next: bool,
    /// Last painted card height per control addr key (Connect tab masonry / waterfall).
    pub(crate) device_masonry_heights: HashMap<String, f32>,
    /// Window management tab: last painted card height per `VmWindowRecord::record_id`.
    pub(crate) vm_window_masonry_heights: HashMap<String, f32>,
    pub(crate) vm_window_create: vm_window_create_dialog::CenterVmWindowCreateForm,
    pub(crate) vm_window_create_id_nonce: u64,
    /// Card overlay delete: applied before painting so the same frame never reads `endpoints[i]` after removal.
    pub(crate) pending_remove_endpoint: Option<usize>,
    /// Window-mgmt card delete: row index into `vm_window_records`; resolved into a `DeleteVmWindowOnHost` RPC.
    pub(crate) pending_delete_vm_window_row_ix: Option<usize>,
    /// Host JSON draft window (device card preview → Configure).
    pub(crate) host_config_window_open: bool,
    /// Draft JSON for [`device_store::host_managed_config`] (host config window).
    pub(crate) host_managed_draft_json: String,
    pub(crate) host_managed_last_msg: String,
    /// Last egui time ([`egui::InputState::time`]) we flushed settings to SQLite (eframe persistence off).
    pub(crate) sqlite_snapshot_last_time: f64,
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
        #[cfg(windows)]
        if titan_tray::consume_windows_tray_quit_close_request() {
            self.really_quitting = true;
            return;
        }
        if let Some(vp) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            vp.events.retain(|e| *e != egui::ViewportEvent::Close);
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        titan_tray::hide_main_window_to_tray(ctx);
        self.hidden_to_tray = true;
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_center_settings_to_sqlite();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.maybe_init_tray_icon_once();
        if let Some(tray) = self._tray.as_ref() {
            titan_tray::sync_tray_if_needed(tray, titan_tray::DesktopProduct::Center, self.ui_lang);
        }
        if let Some(until) = self.ui_toast_until
            && ctx.input(|i| i.time) >= until
        {
            self.ui_toast_until = None;
            self.ui_toast_text.clear();
        }
        if titan_tray::poll_tray_for_egui(ctx, &mut self.really_quitting) {
            self.hidden_to_tray = false;
        }
        if self.hidden_to_tray {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }
        self.center_app_tick_frame(ctx);
        self.center_app_paint_frame(ctx);
    }
}
