//! Manual add-host Hello verification (multi-thread Tokio).

use std::sync::mpsc::Sender;
use std::time::Duration;

use titan_common::ControlResponse;
use tokio::time::timeout;

use super::super::constants::{
    ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS, ADD_HOST_VERIFY_UI_DEADLINE_SECS,
};
use super::super::net::{capabilities_summary, hello_host, NetUiMsg};
use super::super::persist_data::HostEndpoint;
use super::super::CenterApp;

pub(super) fn run_add_host_verify_worker(
    addr: String,
    sid: u64,
    tx: Sender<NetUiMsg>,
    ctx: egui::Context,
) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(NetUiMsg::AddHostVerifyDone {
                session_id: sid,
                addr,
                ok: false,
                device_id: String::new(),
                caps_summary: String::new(),
                error: format!("tokio runtime: {e}"),
            });
            ctx.request_repaint();
            return;
        }
    };
    let msg = map_hello_to_verify_msg(&addr, sid, &rt);
    let _ = tx.send(msg);
    ctx.request_repaint();
}

fn map_hello_to_verify_msg(addr: &str, sid: u64, rt: &tokio::runtime::Runtime) -> NetUiMsg {
    let inner = rt.block_on(timeout(
        Duration::from_secs(ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS),
        hello_host(addr),
    ));
    match inner {
        Ok(Ok(ControlResponse::HelloAck { capabilities })) => {
            verify_done_ok(addr, sid, &capabilities)
        }
        Ok(Ok(ControlResponse::ServerError { code, message })) => {
            verify_done_err(addr, sid, format!("host error {code}: {message}"))
        }
        Ok(Ok(_)) => verify_done_err(addr, sid, "unexpected control response".into()),
        Ok(Err(e)) => verify_done_err(addr, sid, e.to_string()),
        Err(_) => verify_done_err(addr, sid, "timeout".into()),
    }
}

fn verify_done_err(addr: &str, sid: u64, error: String) -> NetUiMsg {
    NetUiMsg::AddHostVerifyDone {
        session_id: sid,
        addr: addr.to_string(),
        ok: false,
        device_id: String::new(),
        caps_summary: String::new(),
        error,
    }
}

fn verify_done_ok(addr: &str, sid: u64, capabilities: &titan_common::Capabilities) -> NetUiMsg {
    let mut did = capabilities.device_id.trim().to_string();
    if did.is_empty() {
        did = HostEndpoint::legacy_device_id_for_addr(addr);
    }
    NetUiMsg::AddHostVerifyDone {
        session_id: sid,
        addr: addr.to_string(),
        ok: true,
        device_id: did,
        caps_summary: capabilities_summary(capabilities),
        error: String::new(),
    }
}

impl CenterApp {
    /// Background Hello to validate the manual add-host address and read [`Capabilities::device_id`].
    ///
    /// Uses a **multi-thread** Tokio runtime so `timeout` + TCP connect advance reliably (the generic
    /// `run_blocking_net` path uses `current_thread` and can stall the timer on some platforms).
    pub(crate) fn spawn_add_host_verify(&mut self, addr: String) {
        if self.add_host_verify_busy {
            return;
        }
        self.add_host_verify_busy = true;
        self.add_host_verify_session = self.add_host_verify_session.wrapping_add(1);
        let sid = self.add_host_verify_session;
        self.add_host_verify_deadline =
            Some(std::time::Instant::now() + Duration::from_secs(ADD_HOST_VERIFY_UI_DEADLINE_SECS));
        self.add_host_dialog_err.clear();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || run_add_host_verify_worker(addr, sid, tx, ctx));
    }
}
