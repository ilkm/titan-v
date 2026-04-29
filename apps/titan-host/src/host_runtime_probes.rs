//! Blocking probes aggregated into [`titan_common::HostRuntimeProbes`] for control-plane capability snapshots.

use titan_common::HostRuntimeProbes;

/// Runs lightweight host probes (driver pipe, etc.). VM / spoof probes align with OpenVMM integration milestones.
#[must_use]
pub fn probe_host_runtime_blocking() -> HostRuntimeProbes {
    HostRuntimeProbes {
        kernel_driver_ipc: crate::driver_bridge::probe_kernel_driver_ipc_blocking(),
        ..Default::default()
    }
}
