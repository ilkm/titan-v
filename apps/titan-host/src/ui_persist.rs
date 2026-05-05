//! Host UI persistence shape (serde) shared by egui and control-plane apply.

use std::net::SocketAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};
use titan_common::{DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT, UiLang};

fn default_vm_storage_root() -> String {
    dirs::home_dir()
        .map(|h| h.join("titan").join("vm").display().to_string())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostUiPersist {
    #[serde(default)]
    pub ui_lang: UiLang,
    pub listen: String,
    pub announce_enabled: bool,
    pub announce_periodic_secs: Option<u64>,
    pub center_register_udp_port: u16,
    pub center_poll_listen_port: u16,
    pub public_addr_override: String,
    pub label_override: String,
    /// Root directory for VM data; each window uses `join(vm_id)` under this path (see settings UI).
    #[serde(default)]
    pub vm_root_directory: String,
}

impl Default for HostUiPersist {
    fn default() -> Self {
        Self {
            ui_lang: UiLang::default(),
            listen: "0.0.0.0:7788".into(),
            announce_enabled: true,
            // Default 1 s safety net so Center can recover even if the initial burst is lost on a
            // congested LAN. Sub-second snap-back relies on `spawn_initial_burst_announce`.
            announce_periodic_secs: Some(1),
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            public_addr_override: String::new(),
            label_override: String::new(),
            vm_root_directory: String::new(),
        }
    }
}

impl HostUiPersist {
    /// Effective VM root: persisted value if non-empty after trim, else `~/titan/vm` when home is known.
    pub fn resolved_vm_root_directory(&self) -> String {
        let s = self.vm_root_directory.trim();
        if s.is_empty() {
            default_vm_storage_root()
        } else {
            s.to_string()
        }
    }

    /// Full path for one VM window: `{root}/{vm_id}` (OS-native separators).
    pub fn vm_directory_for_vm_id(&self, vm_id: u32) -> Option<String> {
        let root = self.resolved_vm_root_directory();
        if root.is_empty() {
            return None;
        }
        let p = Path::new(&root).join(vm_id.to_string());
        Some(p.display().to_string())
    }

    pub fn parse_listen(&self) -> Result<SocketAddr, String> {
        self.listen
            .trim()
            .parse()
            .map_err(|e| format!("invalid listen address: {e}"))
    }

    /// Validates fields needed before enqueueing a remote config apply (no `serve` types).
    pub fn validate_for_remote_apply(&self) -> Result<(), String> {
        self.parse_listen()?;
        Ok(())
    }
}
