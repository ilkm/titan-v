//! Control-plane `Hello` and `ListVms` on the generic current-thread Tokio worker.

use titan_common::{ControlRequest, ControlResponse};

use super::super::CenterApp;
use super::super::net::{NetUiMsg, capabilities_summary, exchange_one, hello_host};
use super::common::run_blocking_net;

impl CenterApp {
    /// Sends `Hello` on the control TCP (periodic auto-connect from the app update loop).
    pub(crate) fn spawn_hello_session(&mut self) {
        if self.net_busy || self.fleet_busy || self.host_connected {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = map_hello_to_net_msg(&addr, rt);
                let _ = tx.send(msg);
            });
        });
    }

    pub(crate) fn spawn_list_vms(&mut self) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = map_list_vms(&addr, rt);
                let _ = tx.send(msg);
            });
        });
    }
}

fn map_hello_to_net_msg(addr: &str, rt: &tokio::runtime::Runtime) -> NetUiMsg {
    match rt.block_on(hello_host(addr)) {
        Ok(ControlResponse::HelloAck { capabilities }) => NetUiMsg::Caps {
            summary: capabilities_summary(&capabilities),
        },
        Ok(ControlResponse::Pong { .. }) => {
            NetUiMsg::Error("unexpected Pong (expected HelloAck); check host version".into())
        }
        Ok(ControlResponse::ServerError { code, message }) => {
            NetUiMsg::Error(format!("host error {code}: {message}"))
        }
        Ok(_) => NetUiMsg::Error("unexpected control response".into()),
        Err(e) => NetUiMsg::Error(e.to_string()),
    }
}

fn map_list_vms(addr: &str, rt: &tokio::runtime::Runtime) -> NetUiMsg {
    match rt.block_on(exchange_one(addr, &ControlRequest::ListVms)) {
        Ok(ControlResponse::VmList { vms }) => NetUiMsg::VmInventory(vms),
        Ok(ControlResponse::ServerError { code, message }) => {
            NetUiMsg::Error(format!("host error {code}: {message}"))
        }
        Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
        Err(e) => NetUiMsg::Error(e.to_string()),
    }
}
