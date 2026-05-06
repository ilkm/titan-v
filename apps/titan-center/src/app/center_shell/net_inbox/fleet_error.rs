use crate::app::CenterApp;
use crate::app::i18n::{self, Msg, t};
use crate::app::net::NetUiMsg;
use crate::app::vm_window_push_to_hosts;

impl CenterApp {
    fn try_dispatch_vm_window_msg(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::VmWindowReloadDone { rows, detail } => {
                self.on_vm_window_reload_done(rows.clone(), detail.clone());
                Some(false)
            }
            NetUiMsg::VmWindowDeleteDone {
                record_id,
                device_id,
                detail,
            } => {
                self.on_vm_window_delete_done(record_id.clone(), device_id.clone(), detail.clone());
                Some(false)
            }
            NetUiMsg::VmWindowRemarkSaveDone {
                record_id: _,
                device_id,
                detail,
            } => {
                self.on_vm_window_remark_save_done(device_id.clone(), detail.clone());
                Some(false)
            }
            NetUiMsg::VmWindowCreatePersistDone { row, error } => {
                self.on_vm_window_create_persist_done(row.clone(), error.clone());
                Some(false)
            }
            _ => None,
        }
    }

    fn try_dispatch_host_config_and_persist_msg(&mut self, msg: &NetUiMsg) -> Option<bool> {
        if let Some(done) = self.try_dispatch_vm_window_msg(msg) {
            return Some(done);
        }
        match msg {
            NetUiMsg::HostConfigLoadDone {
                device_id,
                json,
                detail,
            } => {
                self.on_net_host_config_load_done(device_id.clone(), json.clone(), detail.clone());
                Some(false)
            }
            NetUiMsg::HostConfigSaveDone { device_id, detail } => {
                self.on_net_host_config_save_done(device_id.clone(), detail.clone());
                Some(false)
            }
            NetUiMsg::CenterPersistFlushDone { ok, detail } => {
                self.on_net_center_persist_flush_done(*ok, detail.clone());
                Some(false)
            }
            other => self.try_dispatch_host_ui_push_done(other),
        }
    }

    fn try_dispatch_host_ui_push_done(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::HostUiPushDone { ok, detail } => {
                self.on_net_host_ui_push_done(*ok, detail.clone());
                Some(false)
            }
            _ => None,
        }
    }

    fn on_net_host_config_load_done(
        &mut self,
        device_id: String,
        json: Option<String>,
        detail: String,
    ) {
        if self.current_selected_device_id() == Some(device_id.as_str())
            && let Some(next) = json
        {
            self.host_managed_draft_json = next;
        }
        self.host_managed_last_msg = detail;
        self.ctx.request_repaint();
    }

    fn on_net_host_config_save_done(&mut self, device_id: String, detail: String) {
        if self.current_selected_device_id() == Some(device_id.as_str()) {
            self.host_managed_last_msg = detail;
            self.ctx.request_repaint();
        }
    }

    fn on_net_center_persist_flush_done(&mut self, ok: bool, detail: String) {
        self.sqlite_snapshot_busy = false;
        if !ok {
            tracing::warn!("center persist snapshot worker: {detail}");
        }
    }

    fn on_net_host_ui_push_done(&mut self, ok: bool, detail: String) {
        let s = format!("push ok={ok}: {detail}");
        self.host_managed_last_msg = s.clone();
        self.ui_toast_text = s;
        self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 5.0);
        self.ctx.request_repaint();
    }

    fn on_vm_window_reload_done(
        &mut self,
        rows: Option<Vec<titan_common::VmWindowRecord>>,
        detail: String,
    ) {
        match rows {
            Some(rows) => {
                self.vm_window_records = rows;
                vm_window_push_to_hosts::push_snapshot_to_all(
                    &self.endpoints,
                    &self.vm_window_records,
                );
                self.ctx.request_repaint();
            }
            None => self.last_net_error = detail,
        }
    }

    fn on_vm_window_delete_done(&mut self, record_id: String, device_id: String, detail: String) {
        if !detail.is_empty() {
            self.last_net_error = detail;
            return;
        }
        self.vm_window_records.retain(|r| r.record_id != record_id);
        if self.vm_window_remark_edit_record_id.as_deref() == Some(record_id.as_str()) {
            self.vm_window_remark_edit_record_id = None;
            self.vm_window_remark_edit_focus_next = false;
        }
        vm_window_push_to_hosts::push_snapshot_for_device(
            &self.endpoints,
            &self.vm_window_records,
            &device_id,
        );
        self.ctx.request_repaint();
    }

    fn on_vm_window_remark_save_done(&mut self, device_id: String, detail: String) {
        if !detail.is_empty() {
            self.last_net_error = detail;
            return;
        }
        vm_window_push_to_hosts::push_snapshot_for_device(
            &self.endpoints,
            &self.vm_window_records,
            &device_id,
        );
    }

    fn on_vm_window_create_persist_done(
        &mut self,
        row: Option<titan_common::VmWindowRecord>,
        error: String,
    ) {
        if !error.is_empty() {
            self.vm_window_create.inline_err = error;
            return;
        }
        let Some(row) = row else {
            return;
        };
        let device_id = row.device_id.clone();
        self.vm_window_records.push(row);
        vm_window_push_to_hosts::push_snapshot_for_device(
            &self.endpoints,
            &self.vm_window_records,
            &device_id,
        );
        self.vm_window_create.dialog_open = false;
        self.vm_window_create.device_ix = None;
        self.vm_window_create.vm_id = titan_common::VM_WINDOW_FOLDER_ID_MIN;
        self.vm_window_create.inline_err.clear();
        let now = self.ctx.input(|i| i.time);
        self.ui_toast_text = t(self.ui_lang, Msg::CenterWinMgmtToastCreated).to_string();
        self.ui_toast_until = Some(now + 4.0);
    }

    pub(super) fn dispatch_net_fleet_and_error(&mut self, msg: NetUiMsg) -> bool {
        if let Some(done) = self.try_dispatch_host_config_and_persist_msg(&msg) {
            return done;
        }
        self.dispatch_net_fleet_error_core(msg)
    }

    fn dispatch_net_fleet_error_core(&mut self, msg: NetUiMsg) -> bool {
        match msg {
            NetUiMsg::TelemetryLinkLost {
                host_key,
                session_gen,
            } => self.net_dispatch_telemetry_link_lost(host_key, session_gen),
            NetUiMsg::FleetOpResult {
                host_key,
                ok,
                detail,
            } => self.net_dispatch_fleet_op_result(host_key, ok, detail),
            NetUiMsg::FleetOpDone => self.net_dispatch_fleet_op_done(),
            NetUiMsg::Error(e) => self.net_dispatch_net_error(e),
            _ => false,
        }
    }

    fn net_dispatch_telemetry_link_lost(&mut self, host_key: String, session_gen: u64) -> bool {
        self.on_net_telemetry_link_lost(host_key, session_gen);
        false
    }

    fn net_dispatch_fleet_op_result(&mut self, host_key: String, ok: bool, detail: String) -> bool {
        self.on_net_fleet_op_result(host_key, ok, detail);
        false
    }

    fn net_dispatch_fleet_op_done(&mut self) -> bool {
        self.fleet_busy = false;
        self.ctx.request_repaint();
        false
    }

    fn net_dispatch_net_error(&mut self, e: String) -> bool {
        self.on_net_error(e);
        false
    }

    fn on_net_fleet_op_result(&mut self, host_key: String, ok: bool, detail: String) {
        if !host_key.is_empty() {
            self.last_action = if ok {
                format!("{host_key}: OK")
            } else {
                format!("{host_key}: {detail}")
            };
            if !ok {
                if !self.last_net_error.is_empty() {
                    self.last_net_error.push_str("; ");
                }
                self.last_net_error
                    .push_str(&format!("{host_key}: {detail}"));
            }
        }
        self.ctx.request_repaint();
    }

    fn on_net_error(&mut self, e: String) {
        self.net_busy = false;
        self.last_net_error = e;
        self.last_action = i18n::log_request_failed(self.ui_lang);
        if !self.is_control_telemetry_live() {
            self.mark_control_endpoint_offline();
        }
        self.ctx.request_repaint();
    }
}
