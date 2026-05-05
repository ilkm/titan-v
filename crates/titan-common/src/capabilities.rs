//! Placeholder capability flags for future center–host negotiation.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Fine-grained **host-side** spoof automation probes (host API surface; OpenVMM / automation when wired).
///
/// Reported in [`crate::wire::ControlResponse`] capability snapshots on the control plane; does not imply guest
/// offline edits or kernel drivers.
#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub struct HostSpoofProbeCaps {
    /// Network identity / MAC policy automation appears available.
    #[serde(default)]
    pub network_identity: bool,
    /// VM checkpoint policy automation appears available.
    #[serde(default)]
    pub vm_checkpoint_policy: bool,
    /// VM processor count automation appears available.
    #[serde(default)]
    pub vm_processor_count: bool,
    /// VLAN configuration on synthetic NICs appears available.
    #[serde(default)]
    pub vm_vlan_config: bool,
    /// Expose nested virtualization extensions appears available.
    #[serde(default)]
    pub vm_expose_virtualization_extensions: bool,
    /// Firmware / secure boot template automation appears available.
    #[serde(default)]
    pub vm_firmware_secure_boot: bool,
    /// Guest vTPM enablement automation appears available.
    #[serde(default)]
    pub vm_vtpm: bool,
}

/// Declares which optional subsystems a node supports (center ↔ host negotiation).
#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub struct Capabilities {
    /// OpenVMM-backed (or equivalent) VM management path is wired on the host.
    pub openvmm: bool,
    pub gpu_partition: bool,
    pub streaming: bool,
    /// When true: an input path is available (guest agent and/or future VMBus driver).
    pub vmbus_input: bool,
    /// Umbrella: host-side network identity automation (see [`HostSpoofProbeCaps::network_identity`]).
    pub hardware_spoof: bool,
    /// Cooperative guest TCP agent configured for at least one VM.
    #[serde(default)]
    pub guest_agent: bool,
    /// Host can run capture/stream **precheck** (not full NVENC/WebRTC).
    #[serde(default)]
    pub streaming_precheck: bool,
    /// Fine-grained host spoof probes (Phase 1.x).
    #[serde(default)]
    pub host_spoof_probes: HostSpoofProbeCaps,
    /// Named-pipe / IOCTL bridge to host kernel driver responds (Phase 2+).
    #[serde(default)]
    pub kernel_driver_ipc: bool,
    /// WinHv / hypervisor guest memory path available (Phase 2+).
    #[serde(default)]
    pub winhv_guest_memory: bool,
    /// VMBus synthetic HID injection path available (Phase 2+).
    #[serde(default)]
    pub vmbus_hid: bool,
    /// Full NVENC encode path (not implemented until R5b).
    #[serde(default)]
    pub streaming_nvenc: bool,
    /// WebRTC egress path (not implemented until R5b).
    #[serde(default)]
    pub streaming_webrtc: bool,
    /// WinDivert kernel forward path (not implemented until R5c).
    #[serde(default)]
    pub windivert_forward: bool,
    /// Host-reported notice (e.g. agent-bindings path missing or unreadable at startup).
    #[serde(default)]
    pub host_notice: String,
    /// OS-stable machine id from the host (`machine-uid`); empty on older hosts.
    #[serde(default)]
    pub device_id: String,
}

impl Capabilities {
    /// Conservative defaults for a fresh build (all disabled).
    #[must_use]
    pub fn stub() -> Self {
        Self::default()
    }

    /// Values the host reports on the QUIC control plane; extend with real probes later.
    #[must_use]
    pub fn host_control_plane() -> Self {
        Self::host_control_plane_with_agents(false, false, HostSpoofProbeCaps::default())
    }

    /// Capability snapshot when `titan-host serve` has guest agent bindings and optional probes.
    #[must_use]
    pub fn host_control_plane_with_agents(
        agent_configured: bool,
        gpu_partition_supported: bool,
        spoof_caps: HostSpoofProbeCaps,
    ) -> Self {
        let probes = HostRuntimeProbes {
            spoof_host: spoof_caps,
            ..Default::default()
        };
        Self::from_host_runtime_probes(agent_configured, gpu_partition_supported, &probes)
    }

    /// Builds [`Capabilities`] from blocking probes done at `titan-host serve` startup.
    #[must_use]
    pub fn from_host_runtime_probes(
        agent_configured: bool,
        gpu_partition_supported: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        #[cfg(windows)]
        {
            Self::from_host_runtime_probes_windows(
                agent_configured,
                gpu_partition_supported,
                probes,
            )
        }
        #[cfg(not(windows))]
        {
            Self::from_host_runtime_probes_non_windows(
                agent_configured,
                gpu_partition_supported,
                probes,
            )
        }
    }

    #[cfg(windows)]
    fn from_host_runtime_probes_windows(
        agent_configured: bool,
        gpu_partition_supported: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        let openvmm = probes.openvmm_wired;
        let mut c = Capabilities {
            openvmm,
            streaming_precheck: openvmm,
            gpu_partition: gpu_partition_supported,
            hardware_spoof: probes.spoof_host.network_identity,
            host_spoof_probes: probes.spoof_host.clone(),
            kernel_driver_ipc: probes.kernel_driver_ipc,
            winhv_guest_memory: probes.winhv_guest_memory,
            vmbus_hid: probes.vmbus_hid,
            streaming_nvenc: probes.streaming_nvenc,
            streaming_webrtc: probes.streaming_webrtc,
            windivert_forward: probes.windivert_forward,
            ..Self::default()
        };
        if agent_configured {
            c.guest_agent = true;
            c.vmbus_input = true;
        }
        c
    }

    #[cfg(not(windows))]
    fn from_host_runtime_probes_non_windows(
        agent_configured: bool,
        gpu_partition_supported: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        Capabilities {
            openvmm: probes.openvmm_wired,
            guest_agent: agent_configured,
            vmbus_input: agent_configured,
            gpu_partition: gpu_partition_supported,
            hardware_spoof: probes.spoof_host.network_identity,
            host_spoof_probes: probes.spoof_host.clone(),
            kernel_driver_ipc: probes.kernel_driver_ipc,
            winhv_guest_memory: probes.winhv_guest_memory,
            vmbus_hid: probes.vmbus_hid,
            streaming_nvenc: probes.streaming_nvenc,
            streaming_webrtc: probes.streaming_webrtc,
            windivert_forward: probes.windivert_forward,
            ..Self::default()
        }
    }
}

/// Aggregated blocking probe results for [`Capabilities::from_host_runtime_probes`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostRuntimeProbes {
    pub spoof_host: HostSpoofProbeCaps,
    /// Set true when OpenVMM integration reports a usable VM management path.
    pub openvmm_wired: bool,
    pub kernel_driver_ipc: bool,
    pub winhv_guest_memory: bool,
    pub vmbus_hid: bool,
    pub streaming_nvenc: bool,
    pub streaming_webrtc: bool,
    pub windivert_forward: bool,
}
