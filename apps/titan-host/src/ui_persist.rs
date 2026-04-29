//! Host UI persistence shape (serde) shared by egui and control-plane apply.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use titan_common::{UiLang, DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT};

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
}

impl Default for HostUiPersist {
    fn default() -> Self {
        Self {
            ui_lang: UiLang::default(),
            listen: "0.0.0.0:7788".into(),
            announce_enabled: true,
            announce_periodic_secs: None,
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            public_addr_override: String::new(),
            label_override: String::new(),
        }
    }
}

impl HostUiPersist {
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
