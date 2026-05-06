//! Serialized center UI state (eframe persistence).
//!
//! The **registered device list** (`HostEndpoint` rows) is stored in SQLite; see [`super::device_store`].

use serde::{Deserialize, Serialize};
use titan_common::{
    DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT, DEFAULT_DISCOVERY_UDP_PORT,
};

use super::i18n::UiLang;

/// Left-nav module (persisted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavTab {
    #[default]
    Connect,
    HostsVms,
    Monitor,
    Settings,
    /// Removed tabs (`spoof`, `power`, …) deserialize here; map to [`Connect`](Self::Connect) at load.
    #[serde(other)]
    Legacy,
}

impl NavTab {
    /// Maps persisted [`Legacy`](Self::Legacy) to a real tab for UI and routing.
    #[must_use]
    pub fn normalize(self) -> Self {
        match self {
            Self::Legacy => Self::Connect,
            other => other,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct HostEndpoint {
    pub label: String,
    pub addr: String,
    /// Stable hardware id from the host (`machine-uid`); empty in JSON means derive at persist time.
    #[serde(default)]
    pub device_id: String,
    /// User note on the device management card (local UI only).
    #[serde(default)]
    pub remark: String,
    #[serde(default)]
    pub last_caps: String,
    /// Running VM count when ListVms last succeeded for this host as selected.
    #[serde(default)]
    pub last_vm_count: u32,
    /// Set true after a successful Hello/Ping to this endpoint as the active session target.
    #[serde(default)]
    pub last_known_online: bool,
}

impl HostEndpoint {
    /// Synthetic id for rows without a host-reported hardware id (manual entry or pre-device-id builds).
    #[must_use]
    pub fn legacy_device_id_for_addr(addr: &str) -> String {
        format!("legacy:{}", addr.trim())
    }

    /// Ensures [`Self::device_id`] is non-empty so it can be used as a DB primary key.
    pub fn ensure_device_id(&mut self) {
        if self.device_id.trim().is_empty() {
            self.device_id = Self::legacy_device_id_for_addr(&self.addr);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CenterPersist {
    #[serde(default)]
    pub accounts: Vec<String>,
    #[serde(default)]
    pub proxy_labels: Vec<String>,
    #[serde(default)]
    pub last_script_version: String,
    #[serde(default)]
    pub list_vms_auto_refresh: bool,
    #[serde(default = "default_list_vms_poll_secs")]
    pub list_vms_poll_secs: u32,
    #[serde(default = "default_discovery_broadcast")]
    pub discovery_broadcast: bool,
    #[serde(default = "default_discovery_interval_secs")]
    pub discovery_interval_secs: u32,
    #[serde(default = "default_discovery_udp_port")]
    pub discovery_udp_port: u16,
    /// IPv4 addresses to bind for UDP discovery (multi-homed); empty = OS default route.
    #[serde(default)]
    pub discovery_bind_ipv4s: Vec<String>,
    /// Periodically broadcast [`titan_common::CenterPollBeacon`] so hosts register (UDP).
    #[serde(default = "default_host_collect_broadcast")]
    pub host_collect_broadcast: bool,
    #[serde(default = "default_host_collect_interval_secs")]
    pub host_collect_interval_secs: u32,
    #[serde(default = "default_host_collect_poll_udp_port")]
    pub host_collect_poll_udp_port: u16,
    #[serde(default = "default_host_collect_register_udp_port")]
    pub host_collect_register_udp_port: u16,
    #[serde(default)]
    pub ui_lang: UiLang,
    #[serde(default)]
    pub active_nav: NavTab,
}

pub fn default_list_vms_poll_secs() -> u32 {
    30
}

pub fn default_discovery_broadcast() -> bool {
    true
}

pub fn default_discovery_interval_secs() -> u32 {
    3
}

pub fn default_discovery_udp_port() -> u16 {
    DEFAULT_DISCOVERY_UDP_PORT
}

pub fn default_host_collect_interval_secs() -> u32 {
    4
}

pub fn default_host_collect_poll_udp_port() -> u16 {
    DEFAULT_CENTER_POLL_UDP_PORT
}

pub fn default_host_collect_register_udp_port() -> u16 {
    DEFAULT_CENTER_REGISTER_UDP_PORT
}

pub fn default_host_collect_broadcast() -> bool {
    true
}
