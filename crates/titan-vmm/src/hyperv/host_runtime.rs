//! Hyper-V host runtime with optional **guest agent** bindings for cooperative read/inject.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use titan_common::{Error, Result};
use titan_common::{GpuPartitioner, HardwareSpoofer, StreamEncoder, VmbusInput};

use crate::traits::{InjectInput, PowerControl, ReadMemory};

use super::guest_agent;
use super::HypervBackend;

/// Maps Hyper-V VM name → guest agent `SocketAddr` (concurrent updates from `titan-host serve`).
pub type AgentBindingTable = DashMap<String, SocketAddr>;

/// Cooperative guest path + Hyper-V power control (no paravisor read without agent).
#[derive(Debug, Clone)]
pub struct HypervHostRuntime {
    agents: Arc<AgentBindingTable>,
}

impl Default for HypervHostRuntime {
    fn default() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
        }
    }
}

impl HypervHostRuntime {
    #[must_use]
    pub fn new(agents: Arc<AgentBindingTable>) -> Self {
        Self { agents }
    }

    #[must_use]
    pub fn agents(&self) -> &Arc<AgentBindingTable> {
        &self.agents
    }

    fn agent_addr(&self, vm_id: &str) -> Result<SocketAddr> {
        let key = vm_id.trim();
        self.agents.get(key).map(|e| *e.value()).ok_or_else(|| Error::HyperVRejected {
            message: format!(
                "no guest agent binding for vm '{key}': add [[binding]] in agent-bindings.toml; see titan_vmm::hyperv::guest_agent protocol docs"
            ),
        })
    }
}

impl ReadMemory for HypervHostRuntime {
    fn read_guest_u64(&self, vm_id: &str, guest_addr: u64) -> Result<u64> {
        let addr = self.agent_addr(vm_id)?;
        let rid = format!("mem-{guest_addr:x}");
        guest_agent::read_guest_u64(&addr, vm_id, guest_addr, &rid, Duration::from_secs(8))
    }
}

impl InjectInput for HypervHostRuntime {
    fn inject_mouse_move(&self, vm_id: &str, x: u32, y: u32) -> Result<()> {
        let addr = self.agent_addr(vm_id)?;
        let rid = format!("mouse-{x}-{y}");
        guest_agent::mouse_move(&addr, vm_id, x, y, &rid, Duration::from_secs(5))
    }
}

impl PowerControl for HypervHostRuntime {
    fn start(&self, vm_id: &str) -> Result<()> {
        HypervBackend.start(vm_id)
    }

    fn stop(&self, vm_id: &str) -> Result<()> {
        HypervBackend.stop(vm_id)
    }
}

impl VmbusInput for HypervHostRuntime {
    fn tap(&self, vm_name: &str, x: u32, y: u32) -> Result<()> {
        self.inject_mouse_move(vm_name, x, y)
    }
}

/// GPU-PV assignment via PowerShell ([`super::gpu_pv`]).
#[derive(Debug, Default, Clone, Copy)]
pub struct HypervGpuPartitioner;

impl GpuPartitioner for HypervGpuPartitioner {
    fn assign(&self, vm_name: &str, partition_id: &str) -> Result<()> {
        super::gpu_pv::assign_gpu_partition(vm_name, partition_id)
    }
}

/// Hardware / identity tweaks via Hyper-V cmdlets ([`super::mother_image`]).
#[derive(Debug, Default, Clone, Copy)]
pub struct HypervHardwareSpoofer;

impl HardwareSpoofer for HypervHardwareSpoofer {
    fn apply(&self, vm_name: &str) -> Result<()> {
        super::mother_image::apply_network_spoof_low_risk(vm_name)
    }
}

/// Pre-flight for streaming: VM exists (does not start capture).
#[derive(Debug, Default, Clone, Copy)]
pub struct HypervStreamPrecheck;

impl StreamEncoder for HypervStreamPrecheck {
    fn start_session(&self, vm_name: &str) -> Result<()> {
        #[cfg(windows)]
        {
            if super::vm_exists_blocking(vm_name)? {
                tracing::info!(%vm_name, "stream precheck: VM exists (GraphicsCapture/NVENC not wired)");
                return Ok(());
            }
            return Err(Error::HyperVRejected {
                message: format!("stream precheck: VM '{}' not found", vm_name.trim()),
            });
        }
        #[cfg(not(windows))]
        {
            let _ = vm_name;
            Err(Error::HyperVRejected {
                message: "stream precheck requires Windows".into(),
            })
        }
    }
}
