use crate::app::CenterApp;
use crate::app::device_store;
use crate::app::persist_data::CenterPersist;

impl CenterApp {
    pub(crate) fn persist_snapshot(&self) -> CenterPersist {
        CenterPersist {
            accounts: self.accounts.clone(),
            proxy_labels: self.proxy_labels.clone(),
            last_script_version: self.last_script_version.clone(),
            list_vms_auto_refresh: self.list_vms_auto_refresh,
            list_vms_poll_secs: self.list_vms_poll_secs.max(5),
            discovery_broadcast: self.discovery_broadcast,
            discovery_interval_secs: self.discovery_interval_secs.max(1),
            discovery_udp_port: self.discovery_udp_port,
            discovery_bind_ipv4s: self.discovery_bind_ipv4s.clone(),
            host_collect_broadcast: self.host_collect_broadcast,
            host_collect_interval_secs: self.host_collect_interval_secs.max(1),
            host_collect_poll_udp_port: self.host_collect_poll_udp_port,
            host_collect_register_udp_port: self.host_collect_register_udp_port,
            ui_lang: self.ui_lang,
            active_nav: self.active_nav,
        }
    }

    pub(crate) fn flush_center_settings_to_sqlite(&self) {
        self.persist_registered_devices();
        let snap = self.persist_snapshot();
        let json = match serde_json::to_string(&snap) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("device_store: center persist snapshot serde: {e}");
                return;
            }
        };
        let db_path = device_store::registration_db_path();
        if let Err(e) = device_store::save_center_persist_json(&db_path, &json) {
            tracing::warn!("device_store: center persist {:?}: {e}", db_path);
        }
    }

    pub(crate) fn maybe_flush_center_sqlite(&mut self, ctx: &egui::Context) {
        const PERIOD_SECS: f64 = 10.0;
        let t = ctx.input(|i| i.time);
        if t - self.sqlite_snapshot_last_time < PERIOD_SECS {
            return;
        }
        self.sqlite_snapshot_last_time = t;
        self.flush_center_settings_to_sqlite();
    }
}
