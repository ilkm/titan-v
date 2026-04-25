//! Control-plane exchanges that use the generic current-thread Tokio worker.

use titan_common::{ControlRequest, ControlResponse, VmSpoofProfile};

use super::super::net_client::{capabilities_summary, exchange_one, hello_host};
use super::super::net_msg::NetUiMsg;
use super::super::CenterApp;
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

    pub(crate) fn spawn_spoof_apply(&mut self, dry_run: bool) {
        if self.net_busy || self.fleet_busy || self.control_addr.trim().is_empty() {
            return;
        }
        if !self.host_connected {
            self.last_net_error =
                "Host session not ready (Hello + telemetry); wait for auto-connect or check address."
                    .into();
            return;
        }
        let vm = self.spoof_target_vm.trim().to_string();
        if vm.is_empty() {
            self.last_net_error = "Spoof: enter a VM name.".into();
            return;
        }
        let profile =
            spoof_profile_from_flags(self.spoof_dynamic_mac, self.spoof_disable_checkpoints);
        self.net_busy = true;
        self.last_net_error.clear();
        let addr = self.control_addr.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_spoof_apply_worker(addr, tx, ctx, dry_run, vm, profile);
        });
    }

    pub(crate) fn spawn_stop_vm_group(&mut self, vm_names: Vec<String>) {
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
                let msg = map_stop_vm_group(&addr, rt, vm_names);
                let _ = tx.send(msg);
            });
        });
    }

    pub(crate) fn spawn_start_vm_group(&mut self, vm_names: Vec<String>) {
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
                let msg = map_start_vm_group(&addr, rt, vm_names);
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

fn map_spoof_exchange(addr: &str, rt: &tokio::runtime::Runtime, req: &ControlRequest) -> NetUiMsg {
    match rt.block_on(exchange_one(addr, req)) {
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
    }
}

fn map_stop_vm_group(addr: &str, rt: &tokio::runtime::Runtime, vm_names: Vec<String>) -> NetUiMsg {
    match rt.block_on(exchange_one(
        addr,
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
    }
}

fn spoof_profile_from_flags(dynamic_mac: bool, disable_checkpoints: bool) -> VmSpoofProfile {
    VmSpoofProfile {
        dynamic_mac,
        disable_checkpoints,
        guest_identity_tag: None,
        ..Default::default()
    }
}

fn run_spoof_apply_worker(
    addr: String,
    tx: std::sync::mpsc::Sender<NetUiMsg>,
    ctx: egui::Context,
    dry_run: bool,
    vm: String,
    profile: VmSpoofProfile,
) {
    run_blocking_net(&tx, &ctx, |rt| {
        let req = ControlRequest::ApplySpoofProfile {
            vm_name: vm,
            dry_run,
            spoof: profile,
        };
        let msg = map_spoof_exchange(&addr, rt, &req);
        let _ = tx.send(msg);
    });
}

fn map_start_vm_group(addr: &str, rt: &tokio::runtime::Runtime, vm_names: Vec<String>) -> NetUiMsg {
    match rt.block_on(exchange_one(
        addr,
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
    }
}
