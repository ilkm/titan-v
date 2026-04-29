use std::sync::mpsc as sync_mpsc;
use std::sync::{Arc, Mutex};

use titan_common::{Capabilities, ControlPush, HostRuntimeProbes, UiLang};
use tokio::sync::broadcast;

use crate::agent_binding_table::AgentBindingTable;
use crate::ui_persist::HostUiPersist;

fn host_capability_hint(caps: &Capabilities) -> &'static str {
    if cfg!(windows) && caps.openvmm {
        return "";
    }
    "control-plane: OpenVMM VM stack not wired in this build (ListVms returns empty; batch power disabled)."
}

/// Shared state for one `serve` process (agent bindings + telemetry fan-out).
pub struct ServeState {
    /// Event-driven telemetry fan-out (VM inventory + disk); dedicated TCP subscribers read this.
    pub(super) telemetry_tx: broadcast::Sender<ControlPush>,
    pub agents: Arc<AgentBindingTable>,
    /// Startup notice from agent-bindings load (surfaced in capability `host_notice`).
    pub(super) host_notice: Mutex<String>,
    pub(super) gpu_partition_available: bool,
    pub(super) runtime_probes: HostRuntimeProbes,
    /// When set, control-plane may queue a full [`HostUiPersist`] for the egui thread to apply + restart serve.
    pub(crate) persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
    /// When set, [`titan_common::ControlRequest::SetUiLang`] queues only language (no serve restart).
    pub(crate) lang_apply_tx: Option<sync_mpsc::Sender<UiLang>>,
}

impl ServeState {
    /// Builds state with the given agent map and probe snapshot.
    pub fn new(
        agents: Arc<AgentBindingTable>,
        host_notice: Mutex<String>,
        gpu_partition_available: bool,
        runtime_probes: HostRuntimeProbes,
        persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
        lang_apply_tx: Option<sync_mpsc::Sender<UiLang>>,
    ) -> Self {
        let (telemetry_tx, _) = broadcast::channel(1024);
        Self {
            telemetry_tx,
            agents,
            host_notice,
            gpu_partition_available,
            runtime_probes,
            persist_apply_tx,
            lang_apply_tx,
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

    /// Minimal state for integration tests.
    pub fn for_test() -> Arc<Self> {
        let (telemetry_tx, _) = broadcast::channel(1024);
        Arc::new(Self {
            telemetry_tx,
            agents: Arc::new(AgentBindingTable::new()),
            host_notice: Mutex::new(String::new()),
            gpu_partition_available: false,
            runtime_probes: HostRuntimeProbes::default(),
            persist_apply_tx: None,
            lang_apply_tx: None,
        })
    }
}
