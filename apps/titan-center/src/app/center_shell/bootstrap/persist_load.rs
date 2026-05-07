use std::path::Path;

use crate::app::device_store;
use crate::app::i18n::UiLang;
use crate::app::persist_data::{
    CenterPersist, HostEndpoint, NavTab, default_discovery_interval_secs,
    default_discovery_udp_port, default_host_collect_interval_secs, default_list_vms_poll_secs,
};

pub(super) fn load_center_bootstrap(db_path: &Path) -> CenterPersist {
    let persist_json = device_store::load_center_persist_json(db_path)
        .ok()
        .flatten();
    let persist: CenterPersist = persist_json
        .as_deref()
        .and_then(|j| serde_json::from_str(j).ok())
        .unwrap_or_else(default_center_persist);
    persist
}

pub(super) fn load_endpoints(db_path: &Path) -> Vec<HostEndpoint> {
    let mut endpoints = device_store::load_registered_devices(db_path).unwrap_or_else(|e| {
        tracing::warn!("device_store: load {:?}: {e}", db_path);
        Vec::new()
    });
    for ep in &mut endpoints {
        ep.ensure_device_id();
    }
    endpoints
}

fn default_center_persist() -> CenterPersist {
    CenterPersist {
        accounts: vec!["demo-account-1".into()],
        proxy_labels: vec!["proxy-pool-a".into()],
        last_script_version: String::new(),
        list_vms_auto_refresh: false,
        list_vms_poll_secs: default_list_vms_poll_secs(),
        discovery_broadcast: true,
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
