//! Host UI JSON draft stored in center SQLite + push over control-plane TCP.

use titan_common::ControlRequest;

use super::super::device_store;
use super::super::net_client::exchange_one;
use super::super::net_msg::NetUiMsg;
use super::super::persist_data::HostEndpoint;
use super::super::CenterApp;

fn host_ui_push_exchange(addr: &str, json: String) -> (bool, String) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => return (false, format!("runtime: {e}")),
    };
    let res = rt.block_on(exchange_one(
        addr,
        &ControlRequest::ApplyHostUiPersistJson { json },
    ));
    match res {
        Ok(titan_common::ControlResponse::HostUiPersistAck { ok, detail }) => (ok, detail),
        Ok(titan_common::ControlResponse::ServerError { message, .. }) => (false, message),
        Ok(other) => (false, format!("unexpected response: {other:?}")),
        Err(e) => (false, e.to_string()),
    }
}

impl CenterApp {
    fn host_managed_load_from_db(&mut self, ep: &HostEndpoint) {
        let db = device_store::registration_db_path();
        let mut d = ep.clone();
        d.ensure_device_id();
        self.host_managed_last_msg = match device_store::load_host_managed_config(&db, &d.device_id)
        {
            Ok(Some(j)) => {
                self.host_managed_draft_json = j;
                "Loaded draft from SQLite.".into()
            }
            Ok(None) => "No draft row for this device_id.".into(),
            Err(e) => format!("SQLite: {e}"),
        };
    }

    fn host_managed_save_to_db(&mut self, ep: &HostEndpoint) {
        let db = device_store::registration_db_path();
        let mut d = ep.clone();
        d.ensure_device_id();
        self.host_managed_last_msg = match device_store::upsert_host_managed_config(
            &db,
            &d.device_id,
            &self.host_managed_draft_json,
        ) {
            Ok(()) => "Saved draft to SQLite.".into(),
            Err(e) => format!("SQLite: {e}"),
        };
    }

    fn host_managed_spawn_push_thread(&self, addr: String, json: String) {
        let tx = self.net_tx.clone();
        let _ = std::thread::Builder::new()
            .name("titan-center-host-ui-push".into())
            .spawn(move || {
                let (ok, detail) = host_ui_push_exchange(&addr, json);
                let _ = tx.send(NetUiMsg::HostUiPushDone { ok, detail });
            });
    }

    fn host_managed_group_inner(&mut self, ui: &mut egui::Ui, ep: &HostEndpoint) {
        ui.heading("Host config (SQLite → push)");
        ui.label("Draft JSON is stored in `devices.sqlite` under the selected host device_id.");
        if !self.host_managed_last_msg.is_empty() {
            ui.label(egui::RichText::new(&self.host_managed_last_msg).small());
        }
        ui.horizontal(|ui| {
            if ui.button("Load draft from DB").clicked() {
                self.host_managed_load_from_db(ep);
            }
            if ui.button("Save draft to DB").clicked() {
                self.host_managed_save_to_db(ep);
            }
        });
        ui.text_edit_multiline(&mut self.host_managed_draft_json);
        ui.horizontal(|ui| {
            let can_push = !self.host_managed_draft_json.trim().is_empty();
            if ui
                .add_enabled(can_push, egui::Button::new("Push to selected host"))
                .clicked()
            {
                self.host_managed_spawn_push_thread(
                    ep.addr.clone(),
                    self.host_managed_draft_json.clone(),
                );
            }
        });
    }

    pub(crate) fn host_managed_config_section(&mut self, ui: &mut egui::Ui) {
        if self.endpoints.is_empty() {
            return;
        }
        let idx = self
            .selected_host
            .min(self.endpoints.len().saturating_sub(1));
        let ep = self.endpoints[idx].clone();
        ui.group(|ui| {
            self.host_managed_group_inner(ui, &ep);
        });
    }
}
