use std::path::Path;

use crate::app::device_store;
use crate::app::i18n::UiLang;
use crate::app::persist_data::{
    CenterPersist, HostEndpoint, NavTab, default_discovery_interval_secs,
    default_discovery_udp_port, default_host_collect_interval_secs, default_list_vms_poll_secs,
};

pub(super) fn load_center_bootstrap(db_path: &Path) -> (CenterPersist, Option<Vec<HostEndpoint>>) {
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
    (persist, legacy_eps)
}

pub(super) fn load_or_migrate_endpoints(
    db_path: &Path,
    legacy_eps: Option<Vec<HostEndpoint>>,
) -> Vec<HostEndpoint> {
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
