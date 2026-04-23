//! Blocking probes aggregated into [`titan_common::HostRuntimeProbes`] for M2 capability snapshots.

use titan_common::HostRuntimeProbes;

/// Runs all lightweight host probes (Hyper-V cmdlet surface, driver pipe, vision placeholders).
#[must_use]
pub fn probe_host_runtime_blocking() -> HostRuntimeProbes {
    HostRuntimeProbes {
        spoof_host: titan_vmm::hyperv::mother_image::probe_spoof_host_caps_blocking(),
        kernel_driver_ipc: crate::driver_bridge::probe_kernel_driver_ipc_blocking(),
        winhv_guest_memory: false,
        vmbus_hid: false,
        streaming_nvenc: false,
        streaming_webrtc: false,
        windivert_forward: false,
    }
}
