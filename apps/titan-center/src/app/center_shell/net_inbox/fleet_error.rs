use crate::app::CenterApp;
use crate::app::i18n;
use crate::app::net::NetUiMsg;

impl CenterApp {
    fn on_net_host_ui_push_done(&mut self, ok: bool, detail: String) {
        let s = format!("push ok={ok}: {detail}");
        self.host_managed_last_msg = s.clone();
        self.ui_toast_text = s;
        self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 5.0);
        self.ctx.request_repaint();
    }

    pub(super) fn dispatch_net_fleet_and_error(&mut self, msg: NetUiMsg) -> bool {
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
            NetUiMsg::HostUiPushDone { ok, detail } => {
                self.on_net_host_ui_push_done(ok, detail);
                false
            }
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
        if !self.telemetry_live {
            self.mark_control_endpoint_offline();
        }
        self.ctx.request_repaint();
    }
}
