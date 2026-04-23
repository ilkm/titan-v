//! Background std::thread workers for M2 requests.

use std::sync::mpsc::Sender;

use titan_common::{ControlRequest, ControlResponse, VmSpoofProfile};

use super::net_client::{capabilities_summary, exchange_one, hello_host, ping_host};
use super::net_msg::NetUiMsg;
use super::CenterApp;

fn run_blocking_net(
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
    run: impl FnOnce(&tokio::runtime::Runtime),
) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(NetUiMsg::Error(format!("tokio runtime: {e}")));
            ctx.request_repaint();
            return;
        }
    };
    run(&rt);
    ctx.request_repaint();
}

impl CenterApp {
    pub(super) fn spawn_connect(&mut self) {
        if self.net_busy || self.host_connected {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(hello_host(&addr)) {
                    Ok(ControlResponse::HelloAck { capabilities }) => NetUiMsg::Caps {
                        summary: capabilities_summary(&capabilities),
                    },
                    Ok(ControlResponse::Pong { .. }) => NetUiMsg::Error(
                        "unexpected Pong (expected HelloAck); check host version".into(),
                    ),
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(_) => NetUiMsg::Error("unexpected control response".into()),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_ping(&mut self) {
        if self.net_busy || !self.host_connected {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(ping_host(&addr)) {
                    Ok(ControlResponse::Pong { capabilities }) => NetUiMsg::Caps {
                        summary: capabilities_summary(&capabilities),
                    },
                    Ok(ControlResponse::HelloAck { .. }) => NetUiMsg::Error(
                        "unexpected HelloAck (expected Pong); check host version".into(),
                    ),
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(_) => NetUiMsg::Error("unexpected control response".into()),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_register_guest_agent(&mut self) {
        if self.net_busy || !self.host_connected || self.control_addr.trim().is_empty() {
            return;
        }
        let vm_name = self.agent_register_vm.trim().to_string();
        let guest_agent_addr = self.agent_register_addr.trim().to_string();
        if vm_name.is_empty() || guest_agent_addr.is_empty() {
            self.last_net_error =
                "RegisterGuestAgent: fill VM name and agent address (host-reachable).".into();
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let req = ControlRequest::RegisterGuestAgent {
                    vm_name: vm_name.clone(),
                    guest_agent_addr,
                };
                let msg = match rt.block_on(exchange_one(&addr, &req)) {
                    Ok(ControlResponse::GuestAgentRegisterAck { vm_name: v }) => {
                        NetUiMsg::GuestAgentReg { vm_name: v }
                    }
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_list_vms(&mut self) {
        if self.net_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(&addr, &ControlRequest::ListVms)) {
                    Ok(ControlResponse::VmList { vms }) => NetUiMsg::VmInventory(vms),
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_spoof_apply(&mut self, dry_run: bool) {
        if self.net_busy || self.control_addr.trim().is_empty() {
            return;
        }
        if !self.host_connected {
            self.last_net_error = "Connect (Hello) before ApplySpoofProfile.".into();
            return;
        }
        let vm = self.spoof_target_vm.trim().to_string();
        if vm.is_empty() {
            self.last_net_error = "Spoof: enter a VM name.".into();
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        let profile = VmSpoofProfile {
            dynamic_mac: self.spoof_dynamic_mac,
            disable_checkpoints: self.spoof_disable_checkpoints,
            guest_identity_tag: None,
            ..Default::default()
        };
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let req = ControlRequest::ApplySpoofProfile {
                    vm_name: vm,
                    dry_run,
                    spoof: profile,
                };
                let msg = match rt.block_on(exchange_one(&addr, &req)) {
                    Ok(ControlResponse::SpoofApplyAck {
                        steps_executed,
                        notes,
                        dry_run: dr,
                        ..
                    }) => NetUiMsg::SpoofApply {
                        dry_run: dr,
                        steps: steps_executed,
                        notes,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_stop_vm_group(&mut self, vm_names: Vec<String>) {
        if self.net_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(
                    &addr,
                    &ControlRequest::StopVmGroup { vm_names },
                )) {
                    Ok(ControlResponse::BatchPowerAck {
                        succeeded,
                        failures,
                    }) => NetUiMsg::BatchStop {
                        succeeded,
                        failures,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }

    pub(super) fn spawn_start_vm_group(&mut self, vm_names: Vec<String>) {
        if self.net_busy || self.control_addr.trim().is_empty() {
            return;
        }
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_blocking_net(&tx, &ctx, |rt| {
                let msg = match rt.block_on(exchange_one(
                    &addr,
                    &ControlRequest::StartVmGroup { vm_names },
                )) {
                    Ok(ControlResponse::BatchPowerAck {
                        succeeded,
                        failures,
                    }) => NetUiMsg::BatchStart {
                        succeeded,
                        failures,
                    },
                    Ok(ControlResponse::ServerError { code, message }) => {
                        NetUiMsg::Error(format!("host error {code}: {message}"))
                    }
                    Ok(other) => NetUiMsg::Error(format!("unexpected response: {other:?}")),
                    Err(e) => NetUiMsg::Error(e.to_string()),
                };
                let _ = tx.send(msg);
            });
        });
    }
}
