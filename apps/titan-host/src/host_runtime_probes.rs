//! Blocking probes aggregated into [`titan_common::HostRuntimeProbes`] for control-plane capability snapshots.

use titan_common::HostRuntimeProbes;

#[cfg(windows)]
fn probe_kernel_driver_ipc_blocking() -> bool {
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(r"\\.\pipe\TitanVHostDriver")
    {
        Ok(f) => {
            drop(f);
            true
        }
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn probe_kernel_driver_ipc_blocking() -> bool {
    false
}

/// Runs lightweight host probes (driver pipe, etc.).
#[must_use]
pub fn probe_host_runtime_blocking() -> HostRuntimeProbes {
    HostRuntimeProbes {
        kernel_driver_ipc: probe_kernel_driver_ipc_blocking(),
        ..Default::default()
    }
}
