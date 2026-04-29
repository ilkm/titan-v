//! Host egui app model (persist + serve handle).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;
use tokio::sync::watch;

use titan_common::{HostResourceStats, VmWindowRecord};

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

/// Draft fields for **创建窗口** modal (units: MiB where 1024 MiB ≈ 1 GiB).
#[derive(Debug, Clone)]
pub(crate) struct CreateWindowForm {
    pub(crate) dialog_open: bool,
    pub(crate) cpu_count: u32,
    pub(crate) memory_mib: u32,
    pub(crate) disk_mib: u32,
    pub(crate) vm_directory: String,
    pub(crate) inline_err: String,
}

impl CreateWindowForm {
    pub(crate) fn with_defaults() -> Self {
        Self {
            dialog_open: false,
            cpu_count: 2,
            memory_mib: 4096,
            disk_mib: 131_072,
            vm_directory: default_vm_directory(),
            inline_err: String::new(),
        }
    }
}

/// `{home}/titan/vm/001`, `002`, … — first subdirectory path that does not yet exist on disk.
pub(crate) fn default_vm_directory() -> String {
    dirs::home_dir()
        .as_deref()
        .map(default_vm_directory_under)
        .unwrap_or_default()
}

pub(crate) fn default_vm_directory_under(home: &Path) -> String {
    first_free_vm_slot(&home.join("titan").join("vm"))
}

fn first_free_vm_slot(vm_root: &Path) -> String {
    for idx in 1u32..=999_999 {
        let candidate = vm_root.join(format!("{idx:03}"));
        if !candidate.exists() {
            return candidate.display().to_string();
        }
    }
    vm_root.join("overflow").display().to_string()
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
    pub(crate) create_window: CreateWindowForm,
    /// Status line under **创建窗口** (save / notify errors or success copy).
    pub(crate) window_mgmt_feedback: String,
    /// Local copy of registered VM windows (JSON); same rows as Titan Center SQLite after UDP notify.
    pub(crate) vm_window_records: Vec<VmWindowRecord>,
    /// Window management masonry: last painted height per `VmWindowRecord::record_id`.
    pub(crate) vm_window_masonry_heights: HashMap<String, f32>,
    /// Parity with Titan Center device-card fork (previews not wired on host yet).
    pub(crate) host_desktop_textures: HashMap<String, egui::TextureHandle>,
    pub(crate) host_resource_stats: HashMap<String, HostResourceStats>,
    pub(crate) pending_remove_endpoint: Option<usize>,
}

impl HostApp {
    /// Stub for device-card fork overlay (configure); host window cards stay offline-style.
    pub(crate) fn open_host_config_from_card(&mut self, _card_index: usize) {}
}

#[cfg(test)]
mod default_vm_path_tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{default_vm_directory_under, first_free_vm_slot};

    #[test]
    fn first_free_skips_existing_numeric_dirs() {
        let home = tempdir().unwrap();
        let vm = home.path().join("titan").join("vm");
        fs::create_dir_all(vm.join("001")).unwrap();
        fs::create_dir_all(vm.join("002")).unwrap();
        let got = default_vm_directory_under(home.path());
        assert_eq!(PathBuf::from(got), vm.join("003"));
    }

    #[test]
    fn first_free_returns_001_when_vm_root_empty() {
        let home = tempdir().unwrap();
        let vm = home.path().join("titan").join("vm");
        let got = first_free_vm_slot(&vm);
        assert_eq!(PathBuf::from(got), vm.join("001"));
    }
}
