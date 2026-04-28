//! Push [`titan_common::ControlRequest::SetUiLang`] to every registered host (fire-and-forget).

use std::thread;

use egui::Context;
use titan_common::{ControlRequest, ControlResponse, UiLang};
use tokio::runtime::Builder;

use super::super::net_client::exchange_one;
use super::super::CenterApp;

impl CenterApp {
    fn endpoint_control_addrs_nonempty(&self) -> Vec<String> {
        self.endpoints
            .iter()
            .map(|e| e.addr.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub(crate) fn tick_sync_ui_lang_to_hosts_if_needed(&mut self) {
        if self.host_synced_ui_lang == self.ui_lang {
            return;
        }
        let lang = self.ui_lang;
        let addrs = self.endpoint_control_addrs_nonempty();
        self.host_synced_ui_lang = lang;
        spawn_ui_lang_push_addresses(self.ctx.clone(), addrs, lang);
    }

    /// After a host is reachable or newly registered, push the center UI language to its control TCP.
    pub(crate) fn spawn_ui_lang_push_to_host_control_addr(&self, control_addr: &str) {
        let a = control_addr.trim().to_string();
        if a.is_empty() {
            return;
        }
        spawn_ui_lang_push_addresses(self.ctx.clone(), vec![a], self.ui_lang);
    }
}

fn spawn_ui_lang_push_addresses(ctx: Context, addrs: Vec<String>, lang: UiLang) {
    thread::spawn(move || ui_lang_sync_worker(ctx, addrs, lang));
}

fn ui_lang_sync_worker(ctx: Context, addrs: Vec<String>, lang: UiLang) {
    let Ok(rt) = Builder::new_current_thread().enable_all().build() else {
        tracing::warn!("ui_lang host sync: failed to build tokio runtime");
        return;
    };
    for addr in addrs {
        let trimmed = addr.trim();
        if trimmed.is_empty() {
            continue;
        }
        let res = rt.block_on(exchange_one(trimmed, &ControlRequest::SetUiLang { lang }));
        log_set_ui_lang_result(trimmed, res);
    }
    ctx.request_repaint();
}

fn log_set_ui_lang_result(addr: &str, res: anyhow::Result<ControlResponse>) {
    match res {
        Ok(ControlResponse::SetUiLangAck { ok: true }) => {}
        Ok(ControlResponse::SetUiLangAck { ok: false }) => {
            tracing::debug!(%addr, "SetUiLangAck returned ok=false");
        }
        Ok(ControlResponse::ServerError { code, message }) => {
            tracing::debug!(%addr, code, %message, "SetUiLang server error");
        }
        Ok(other) => tracing::debug!(%addr, ?other, "SetUiLang unexpected response"),
        Err(e) => tracing::debug!(%addr, error = %e, "SetUiLang exchange failed"),
    }
}
