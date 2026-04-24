use std::sync::{Arc, Mutex, OnceLock};

use dashmap::DashMap;
use titan_common::{Capabilities, ControlPush, ControlResponse, HostRuntimeProbes};
use titan_vmm::hyperv::AgentBindingTable;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;

use crate::runtime;
use crate::runtime::ScriptJob;

use super::response::server_err;

fn host_capability_hint(caps: &Capabilities) -> &'static str {
    if cfg!(windows) {
        if caps.hyperv {
            return "";
        }
        return "hyperv: Hyper-V role / PowerShell module not detected on this Windows host.";
    }
    if cfg!(target_os = "linux") {
        if caps.linux_virsh_inventory {
            return "linux: VM list and batch power use virsh (libvirt shell); Hyper-V is N/A on Linux.";
        }
        return "linux: virsh not on PATH — ListVms returns empty and batch power is unavailable until libvirt client tools are installed.";
    }
    if cfg!(target_os = "macos") {
        return "macos: VM inventory and batch power are not implemented yet (Virtualization.framework path pending).";
    }
    ""
}

/// Shared state for one `serve` process (agent bindings + script queue).
#[derive(Debug)]
pub struct ServeState {
    /// Event-driven telemetry fan-out (VM inventory + disk); dedicated TCP subscribers read this.
    pub(super) telemetry_tx: broadcast::Sender<ControlPush>,
    pub agents: Arc<AgentBindingTable>,
    /// Startup notice from agent-bindings load (surfaced in capability `host_notice`).
    pub(super) host_notice: Mutex<String>,
    pub(super) script_tx: mpsc::Sender<ScriptJob>,
    pub(super) gpu_partition_available: bool,
    pub(super) runtime_probes: HostRuntimeProbes,
}

impl ServeState {
    /// Builds state with the given agent map and an already-created script sender.
    pub fn new(
        agents: Arc<AgentBindingTable>,
        host_notice: Mutex<String>,
        script_tx: mpsc::Sender<ScriptJob>,
        gpu_partition_available: bool,
        runtime_probes: HostRuntimeProbes,
    ) -> Self {
        let (telemetry_tx, _) = broadcast::channel(1024);
        Self {
            telemetry_tx,
            agents,
            host_notice,
            script_tx,
            gpu_partition_available,
            runtime_probes,
        }
    }

    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        let mut c = Capabilities::from_host_runtime_probes(
            !self.agents.is_empty(),
            self.gpu_partition_available,
            &self.runtime_probes,
        );
        let mut note = self
            .host_notice
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        let hint = host_capability_hint(&c);
        if !hint.is_empty() {
            if !note.is_empty() {
                note.push_str("; ");
            }
            note.push_str(hint);
        }
        c.host_notice = note;
        c.device_id = crate::host_device_id::host_device_id_string();
        c
    }

    /// Minimal state for integration tests (starts a script worker).
    pub fn for_test() -> Arc<Self> {
        let (tx, rx) = mpsc::channel(8);
        let vm_locks = Arc::new(DashMap::<String, Arc<AsyncMutex<()>>>::new());
        tokio::spawn(runtime::script_worker(rx, vm_locks));
        let (telemetry_tx, _) = broadcast::channel(1024);
        Arc::new(Self {
            telemetry_tx,
            agents: Arc::new(AgentBindingTable::new()),
            host_notice: Mutex::new(String::new()),
            script_tx: tx,
            gpu_partition_available: false,
            runtime_probes: HostRuntimeProbes::default(),
        })
    }
}

pub(super) fn script_artifact_cell() -> &'static Mutex<Option<(String, String)>> {
    static CELL: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

/// Enqueues a Lua script job; used by [`super::dispatch::dispatch_request`] and unit-tested for backpressure.
pub(super) fn try_enqueue_script_vm(
    script_tx: &mpsc::Sender<ScriptJob>,
    vm_name: String,
    source: String,
) -> ControlResponse {
    match script_tx.try_send(ScriptJob {
        vm_name: vm_name.clone(),
        source,
    }) {
        Ok(()) => ControlResponse::ScriptLoadAck { vm_name },
        Err(mpsc::error::TrySendError::Full(_)) => {
            server_err(503, "script execution queue is full; retry later")
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            server_err(503, "script worker is not available")
        }
    }
}

#[cfg(test)]
mod script_enqueue_tests {
    use super::*;
    use crate::runtime::ScriptJob;

    #[test]
    fn load_script_queue_full_returns_503() {
        let (tx, _rx) = mpsc::channel(1);
        tx.try_send(ScriptJob {
            vm_name: "a".into(),
            source: "1".into(),
        })
        .unwrap();
        let r = try_enqueue_script_vm(&tx, "b".into(), "2".into());
        match r {
            ControlResponse::ServerError { code, .. } => assert_eq!(code, 503),
            _ => panic!("unexpected {r:?}"),
        }
    }

    #[test]
    fn load_script_queue_closed_returns_503() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        let r = try_enqueue_script_vm(&tx, "b".into(), "2".into());
        match r {
            ControlResponse::ServerError { code, .. } => assert_eq!(code, 503),
            _ => panic!("unexpected {r:?}"),
        }
    }

    #[test]
    fn load_script_ok_returns_ack() {
        let (tx, _rx) = mpsc::channel(4);
        let r = try_enqueue_script_vm(&tx, "vm1".into(), "return 1".into());
        match r {
            ControlResponse::ScriptLoadAck { vm_name } => assert_eq!(vm_name, "vm1"),
            _ => panic!("unexpected {r:?}"),
        }
    }
}
