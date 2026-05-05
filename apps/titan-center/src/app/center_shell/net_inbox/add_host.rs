use crate::app::CenterApp;
use crate::app::i18n::{self, Msg};
use crate::app::net::NetUiMsg;

impl CenterApp {
    pub(super) fn try_net_add_host_verify_only(&mut self, msg: &NetUiMsg) -> Option<bool> {
        let NetUiMsg::AddHostVerifyDone {
            session_id,
            addr,
            ok,
            device_id,
            caps_summary,
            error,
        } = msg
        else {
            return None;
        };
        Some(self.on_net_add_host_verify_done(
            *session_id,
            addr.clone(),
            *ok,
            device_id.clone(),
            caps_summary.clone(),
            error.clone(),
        ))
    }

    fn on_net_add_host_verify_done(
        &mut self,
        session_id: u64,
        addr: String,
        ok: bool,
        device_id: String,
        caps_summary: String,
        error: String,
    ) -> bool {
        if session_id != self.add_host_verify_session {
            return true;
        }
        self.add_host_verify_busy = false;
        self.add_host_verify_deadline = None;
        if ok {
            self.add_host_verify_succeeded(addr, device_id, caps_summary);
        } else {
            self.add_host_verify_failed(&addr, &error);
        }
        self.ctx.request_repaint();
        false
    }

    fn add_host_verify_succeeded(&mut self, addr: String, device_id: String, caps: String) {
        self.merge_add_host_after_verify(addr.clone(), device_id, caps);
        self.add_host_dialog_open = false;
        self.add_host_dialog_err.clear();
        self.persist_registered_devices();
        self.last_net_error.clear();
        self.last_action = i18n::t(self.ui_lang, Msg::AddHostSavedLog).to_string();
        self.spawn_ui_lang_push_to_host_control_addr(&addr);
    }

    fn add_host_verify_failed(&mut self, addr: &str, error: &str) {
        if self.maybe_raise_tofu_for_verify_error(addr, error) {
            tracing::debug!(%addr, %error, "add host: untrusted fingerprint → TOFU prompt");
            return;
        }
        tracing::debug!(%addr, %error, "add host: Hello verify failed");
        self.ui_toast_text = i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
        self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 3.8);
    }
}
