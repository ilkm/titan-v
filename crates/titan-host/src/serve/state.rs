use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use dashmap::DashMap;
use titan_common::{Capabilities, ControlResponse, HostRuntimeProbes};
use titan_vmm::hyperv::AgentBindingTable;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;

use crate::runtime;
use crate::runtime::ScriptJob;

use super::response::server_err;

/// Shared state for one `serve` process (agent bindings + script queue).
#[derive(Debug)]
pub struct ServeState {
    pub agents: Arc<AgentBindingTable>,
    /// When set, successful [`titan_common::ControlRequest::RegisterGuestAgent`] is written back to disk.
    pub(super) agent_bindings_path: Option<PathBuf>,
    pub(super) script_tx: mpsc::Sender<ScriptJob>,
    pub(super) gpu_partition_available: bool,
    pub(super) runtime_probes: HostRuntimeProbes,
}

impl ServeState {
    /// Builds state with the given agent map and an already-created script sender.
    pub fn new(
        agents: Arc<AgentBindingTable>,
        agent_bindings_path: Option<PathBuf>,
        script_tx: mpsc::Sender<ScriptJob>,
        gpu_partition_available: bool,
        runtime_probes: HostRuntimeProbes,
    ) -> Self {
        Self {
            agents,
            agent_bindings_path,
            script_tx,
            gpu_partition_available,
            runtime_probes,
        }
    }

    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        Capabilities::from_host_runtime_probes(
            !self.agents.is_empty(),
            self.gpu_partition_available,
            &self.runtime_probes,
        )
    }

    /// Minimal state for integration tests (starts a script worker).
    pub fn for_test() -> Arc<Self> {
        let (tx, rx) = mpsc::channel(8);
        let vm_locks = Arc::new(DashMap::<String, Arc<AsyncMutex<()>>>::new());
        tokio::spawn(runtime::script_worker(rx, vm_locks));
        Arc::new(Self {
            agents: Arc::new(AgentBindingTable::new()),
            agent_bindings_path: None,
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
