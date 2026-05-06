//! Host egui app model (persist + serve handle).

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;
use tokio::sync::watch;

use titan_common::{HostResourceStats, VmWindowRecord};
use titan_quic::{Identity, Pairing, TrustStore};

use crate::agent_binding_table::AgentBindingTable;
use crate::serve::{HostAnnounceConfig, VmWindowReloadMsg};

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
            bind_ipv4: parse_bind_ipv4(&self.lan_bind_ipv4),
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

fn parse_bind_ipv4(raw: &str) -> Option<Ipv4Addr> {
    let s = raw.trim();
    if s.is_empty() {
        None
    } else {
        s.parse::<Ipv4Addr>().ok()
    }
}

pub(crate) struct ServeRun {
    pub(crate) shutdown_tx: watch::Sender<bool>,
    pub(crate) join: JoinHandle<()>,
}

impl ServeRun {
    pub(crate) fn stop(self) {
        if let Err(e) = self.shutdown_tx.send(true) {
            tracing::debug!(error = %e, "host serve shutdown signal failed");
        }
        let _ = std::thread::Builder::new()
            .name("titan-host-serve-join".into())
            .spawn(move || {
                let _ = self.join.join();
            });
    }
}

/// mTLS identity, fingerprint trust list, and pairing-window state for the host's QUIC server.
pub struct HostSecurity {
    pub identity: Arc<Identity>,
    pub trust: Arc<TrustStore>,
    pub pairing: Arc<Pairing>,
}

pub struct HostApp {
    pub(crate) host_security: HostSecurity,
    pub(crate) really_quitting: bool,
    pub(crate) hidden_to_tray: bool,
    pub(crate) _tray: Option<titan_tray::TrayIcon>,
    pub(crate) serve_run: Option<ServeRun>,
    pub(crate) persist_apply_tx: Option<std::sync::mpsc::Sender<HostUiPersist>>,
    pub(crate) persist_apply_rx: std::sync::mpsc::Receiver<HostUiPersist>,
    pub(crate) lang_apply_tx: Option<std::sync::mpsc::Sender<titan_common::UiLang>>,
    pub(crate) lang_apply_rx: std::sync::mpsc::Receiver<titan_common::UiLang>,
    /// Serve thread → egui: VM-window mutations from Titan Center (upsert / authoritative snapshot).
    pub(crate) vm_windows_reload_tx: Option<std::sync::mpsc::Sender<VmWindowReloadMsg>>,
    pub(crate) vm_windows_reload_rx: std::sync::mpsc::Receiver<VmWindowReloadMsg>,
    pub(crate) persist: HostUiPersist,
    pub(crate) active_tab: usize,
    pub(crate) status_line: String,
    pub(crate) env_listen_hint: Option<String>,
    /// First `update` tick starts serve once (invalid listen → user fixes and clicks restart).
    pub(crate) initial_serve_attempted: bool,
    /// One-shot: bring the native window to front after eframe's initial `with_visible(false)` bootstrap.
    pub(crate) boot_window_focus_once: bool,
    /// VM-window rows owned by this host (host SQLite is the single source of truth).
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
use std::path::Path;

#[cfg(test)]
fn first_free_vm_slot(vm_root: &Path) -> String {
    for idx in 1u32..=999_999 {
        let candidate = vm_root.join(format!("{idx:03}"));
        if !candidate.exists() {
            return candidate.display().to_string();
        }
    }
    vm_root.join("overflow").display().to_string()
}

/// `{home}/titan/vm/001`, `002`, … — first free subdirectory (tests only; host UI uses settings root + numeric id).
#[cfg(test)]
pub(crate) fn default_vm_directory_under(home: &Path) -> String {
    first_free_vm_slot(&home.join("titan").join("vm"))
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
