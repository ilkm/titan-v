//! Host egui app model (persist + serve handle).

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;
use tokio::sync::watch;

use crate::agent_binding_table::AgentBindingTable;
use crate::serve::HostAnnounceConfig;

pub const PERSIST_KEY: &str = "titan_host_ui_v1";

pub use crate::ui_persist::HostUiPersist;

impl HostUiPersist {
    pub(crate) fn to_announce(&self) -> HostAnnounceConfig {
        HostAnnounceConfig {
            enabled: self.announce_enabled,
            periodic_interval: self
                .announce_periodic_secs
                .filter(|&s| s > 0)
                .map(Duration::from_secs),
            center_register_udp_port: self.center_register_udp_port,
            center_poll_listen_port: self.center_poll_listen_port,
            public_addr_override: {
                let s = self.public_addr_override.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            },
            label_override: {
                let s = self.label_override.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            },
        }
    }

    /// In-memory empty VM→agent table (no on-disk `agent-bindings.toml` in this build).
    pub(crate) fn agent_bindings_for_serve() -> (Arc<AgentBindingTable>, String) {
        (Arc::new(AgentBindingTable::new()), String::new())
    }
}

pub(crate) struct ServeRun {
    pub(crate) shutdown_tx: watch::Sender<bool>,
    pub(crate) join: JoinHandle<()>,
}

impl ServeRun {
    pub(crate) fn stop(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.join.join();
    }
}

pub struct HostApp {
    pub(crate) really_quitting: bool,
    pub(crate) hidden_to_tray: bool,
    pub(crate) _tray: Option<titan_tray::TrayIcon>,
    /// Last UI language applied to the tray (icon, menu, tooltip); see [`titan_tray::refresh_tray_icon`].
    pub(crate) tray_glyph_lang: titan_common::UiLang,
    pub(crate) serve_run: Option<ServeRun>,
    pub(crate) persist_apply_tx: Option<std::sync::mpsc::Sender<HostUiPersist>>,
    pub(crate) persist_apply_rx: std::sync::mpsc::Receiver<HostUiPersist>,
    pub(crate) lang_apply_tx: Option<std::sync::mpsc::Sender<titan_common::UiLang>>,
    pub(crate) lang_apply_rx: std::sync::mpsc::Receiver<titan_common::UiLang>,
    pub(crate) persist: HostUiPersist,
    pub(crate) active_tab: usize,
    pub(crate) status_line: String,
    pub(crate) env_listen_hint: Option<String>,
    /// First `update` tick starts serve once (invalid listen → user fixes and clicks restart).
    pub(crate) initial_serve_attempted: bool,
    /// One-shot: bring the native window to front after eframe's initial `with_visible(false)` bootstrap.
    pub(crate) boot_window_focus_once: bool,
    pub(crate) settings_open: bool,
    /// Last frame's 🌐 button rect (screen space); anchors the language popup like Titan Center.
    pub(crate) settings_lang_btn_rect: Option<egui::Rect>,
}
