//! Serialized center UI state (eframe persistence).

use serde::{Deserialize, Serialize};
use titan_common::DEFAULT_DISCOVERY_UDP_PORT;

use super::i18n::UiLang;

/// Left-nav module (persisted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum NavTab {
    #[default]
    Connect,
    HostsVms,
    Monitor,
    Spoof,
    Power,
    Settings,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct HostEndpoint {
    pub label: String,
    pub addr: String,
    #[serde(default)]
    pub last_caps: String,
    /// Running VM count when ListVms last succeeded for this host as selected.
    #[serde(default)]
    pub last_vm_count: u32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CenterPersist {
    #[serde(default)]
    pub endpoints: Vec<HostEndpoint>,
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
    #[serde(default)]
    pub discovery_broadcast: bool,
    #[serde(default = "default_discovery_interval_secs")]
    pub discovery_interval_secs: u32,
    #[serde(default = "default_discovery_udp_port")]
    pub discovery_udp_port: u16,
    #[serde(default)]
    pub ui_lang: UiLang,
    #[serde(default)]
    pub active_nav: NavTab,
}

pub fn default_list_vms_poll_secs() -> u32 {
    30
}

pub fn default_discovery_interval_secs() -> u32 {
    5
}

pub fn default_discovery_udp_port() -> u16 {
    DEFAULT_DISCOVERY_UDP_PORT
}
