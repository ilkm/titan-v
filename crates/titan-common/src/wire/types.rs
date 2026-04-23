//! Control-plane message types (postcard-serialized bodies).

use serde::{Deserialize, Serialize};

use crate::capabilities::Capabilities;
use crate::plan::VmSpoofProfile;
use crate::state::VmPowerState;

/// Center → host control request.
///
/// **Wire stability**: new variants append at the end; bump [`crate::PROTOCOL_VERSION`] when
/// breaking layout is unavoidable. Postcard discriminant follows declaration order (`Ping` = 0).
///
/// Guest memory / mouse uses the separate JSON guest-agent TCP protocol. **Registration** of the
/// guest agent address for a VM is [`ControlRequest::RegisterGuestAgent`] on the host M2 socket
/// (typically **center → host** or another operator client; host persists bindings).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlRequest {
    /// Liveness + capability snapshot (same payload shape as [`ControlRequest::Hello`] response).
    Ping,
    /// Session handshake; host replies with [`ControlResponse::HelloAck`].
    Hello,
    /// Enumerate VMs on the host (Hyper-V `Get-VM` on Windows; empty elsewhere).
    ListVms,
    /// Start each named VM (best-effort; see [`ControlResponse::BatchPowerAck`]).
    StartVmGroup { vm_names: Vec<String> },
    /// Stop each named VM (best-effort).
    StopVmGroup { vm_names: Vec<String> },
    /// Record script artifact metadata for a later load path (no large body on wire).
    SetScriptArtifact { version: String, sha256_hex: String },
    /// Load or replace a per-VM Lua chunk and execute it once (bounded by host policy).
    LoadScriptVm { vm_name: String, source: String },
    /// Guest or operator registers **Hyper-V VM name → guest agent TCP** on the host (scheme A).
    RegisterGuestAgent {
        vm_name: String,
        /// Guest agent listen address **as seen from the host** (e.g. `192.168.1.50:9000`).
        guest_agent_addr: String,
    },
    /// Apply host-side [`VmSpoofProfile`] steps to an existing VM (PowerShell; Windows).
    ApplySpoofProfile {
        vm_name: String,
        dry_run: bool,
        spoof: VmSpoofProfile,
    },
    /// Apply a single spoof step by id (see `titan_vmm::hyperv::mother_image` step names).
    ApplySpoofStep {
        vm_name: String,
        step_id: String,
        dry_run: bool,
    },
}

/// One row in a [`ControlResponse::VmList`] payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmBrief {
    pub name: String,
    pub state: VmPowerState,
}

/// Host → center response.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlResponse {
    Pong {
        capabilities: Capabilities,
    },
    /// Answer to [`ControlRequest::Hello`].
    HelloAck {
        capabilities: Capabilities,
    },
    /// Recoverable protocol or application-level failure on the host.
    ServerError {
        code: u16,
        message: String,
    },
    /// Answer to [`ControlRequest::ListVms`].
    VmList {
        vms: Vec<VmBrief>,
    },
    /// Result of [`ControlRequest::StartVmGroup`] / [`ControlRequest::StopVmGroup`].
    BatchPowerAck {
        succeeded: u32,
        failures: Vec<String>,
    },
    /// Script metadata stored (echoes accepted version).
    ScriptArtifactAck {
        version: String,
    },
    /// Script was accepted and executed for the VM (or queued on the runtime).
    ScriptLoadAck {
        vm_name: String,
    },
    /// [`ControlRequest::RegisterGuestAgent`] applied; host will use this binding for guest-agent RPC.
    GuestAgentRegisterAck {
        vm_name: String,
    },
    /// Result of [`ControlRequest::ApplySpoofProfile`].
    SpoofApplyAck {
        vm_name: String,
        dry_run: bool,
        steps_executed: Vec<String>,
        notes: String,
    },
    /// Result of [`ControlRequest::ApplySpoofStep`].
    SpoofStepAck {
        vm_name: String,
        step_id: String,
        dry_run: bool,
        ok: bool,
        detail: String,
    },
}
