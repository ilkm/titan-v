//! Placeholder capability flags for future center–host negotiation.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Hyper-V–only **host-side** spoof automation probes (PowerShell cmdlet surface).
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
pub struct HypervSpoofHostCaps {
    /// `Get-VMNetworkAdapter` / `Set-VMNetworkAdapter` (dynamic MAC path).
    #[serde(default)]
    pub network_identity: bool,
    /// `Set-VM -CheckpointType` (or equivalent) appears available.
    #[serde(default)]
    pub vm_checkpoint_policy: bool,
    /// `Set-VM -ProcessorCount` appears available.
    #[serde(default)]
    pub vm_processor_count: bool,
    /// `Set-VMNetworkAdapterVlanConfiguration` appears available.
    #[serde(default)]
    pub vm_vlan_config: bool,
    /// `Set-VMProcessor -ExposeVirtualizationExtensions` appears available.
    #[serde(default)]
    pub vm_expose_virtualization_extensions: bool,
    /// `Set-VMFirmware` / secure boot template path appears available.
    #[serde(default)]
    pub vm_firmware_secure_boot: bool,
    /// vTPM cmdlets appear available.
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
    pub hyperv: bool,
    pub gpu_partition: bool,
    pub streaming: bool,
    /// When true: an input path is available (guest agent and/or future VMBus driver).
    pub vmbus_input: bool,
    /// Umbrella: host-side network identity cmdlets (see [`HypervSpoofHostCaps::network_identity`]).
    pub hardware_spoof: bool,
    /// Cooperative guest TCP agent configured for at least one VM.
    #[serde(default)]
    pub guest_agent: bool,
    /// Host can run capture/stream **precheck** (not full NVENC/WebRTC).
    #[serde(default)]
    pub streaming_precheck: bool,
    /// Fine-grained Hyper-V host spoof probes (Phase 1.x).
    #[serde(default)]
    pub hyperv_spoof_host: HypervSpoofHostCaps,
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
    /// Linux: `virsh` on PATH for optional list / batch power (libvirt shell; not full QEMU parity).
    #[serde(default)]
    pub linux_virsh_inventory: bool,
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

    /// Values the host reports on the control TCP socket; extend with real probes later.
    #[must_use]
    pub fn host_control_plane() -> Self {
        Self::host_control_plane_with_agents(false, false, HypervSpoofHostCaps::default())
    }

    /// Capability snapshot when `titan-host serve` has guest agent bindings and optional probes.
    #[must_use]
    pub fn host_control_plane_with_agents(
        agent_configured: bool,
        gpu_partition_cmdlets_available: bool,
        spoof_host_caps: HypervSpoofHostCaps,
    ) -> Self {
        let probes = HostRuntimeProbes {
            spoof_host: spoof_host_caps,
            ..Default::default()
        };
        Self::from_host_runtime_probes(agent_configured, gpu_partition_cmdlets_available, &probes)
    }

    /// Builds [`Capabilities`] from blocking probes done at `titan-host serve` startup.
    #[must_use]
    pub fn from_host_runtime_probes(
        agent_configured: bool,
        gpu_partition_cmdlets_available: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        #[cfg(windows)]
        {
            Self::from_host_runtime_probes_windows(
                agent_configured,
                gpu_partition_cmdlets_available,
                probes,
            )
        }
        #[cfg(not(windows))]
        {
            Self::from_host_runtime_probes_non_windows(
                agent_configured,
                gpu_partition_cmdlets_available,
                probes,
            )
        }
    }

    #[cfg(windows)]
    fn from_host_runtime_probes_windows(
        agent_configured: bool,
        gpu_partition_cmdlets_available: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        let hv = probes.hyperv_ps_module_available;
        let mut c = Capabilities {
            hyperv: hv,
            streaming_precheck: hv,
            gpu_partition: gpu_partition_cmdlets_available,
            hardware_spoof: probes.spoof_host.network_identity,
            hyperv_spoof_host: probes.spoof_host.clone(),
            kernel_driver_ipc: probes.kernel_driver_ipc,
            winhv_guest_memory: probes.winhv_guest_memory,
            vmbus_hid: probes.vmbus_hid,
            streaming_nvenc: probes.streaming_nvenc,
            streaming_webrtc: probes.streaming_webrtc,
            windivert_forward: probes.windivert_forward,
            linux_virsh_inventory: false,
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
        gpu_partition_cmdlets_available: bool,
        probes: &HostRuntimeProbes,
    ) -> Self {
        Capabilities {
            hyperv: probes.hyperv_ps_module_available,
            guest_agent: agent_configured,
            vmbus_input: agent_configured,
            gpu_partition: gpu_partition_cmdlets_available,
            hardware_spoof: probes.spoof_host.network_identity,
            hyperv_spoof_host: probes.spoof_host.clone(),
            kernel_driver_ipc: probes.kernel_driver_ipc,
            winhv_guest_memory: probes.winhv_guest_memory,
            vmbus_hid: probes.vmbus_hid,
            streaming_nvenc: probes.streaming_nvenc,
            streaming_webrtc: probes.streaming_webrtc,
            windivert_forward: probes.windivert_forward,
            linux_virsh_inventory: probes.linux_virsh_available,
            ..Default::default()
        }
    }
}

/// Aggregated blocking probe results for [`Capabilities::from_host_runtime_probes`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostRuntimeProbes {
    pub spoof_host: HypervSpoofHostCaps,
    /// `Import-Module Hyper-V` / module list probe (Windows); always false off-Windows.
    pub hyperv_ps_module_available: bool,
    pub kernel_driver_ipc: bool,
    pub winhv_guest_memory: bool,
    pub vmbus_hid: bool,
    pub streaming_nvenc: bool,
    pub streaming_webrtc: bool,
    pub windivert_forward: bool,
    /// Linux: `virsh --version` succeeds (libvirt client tools).
    pub linux_virsh_available: bool,
}
