//! Control-plane message types (rkyv-serialized bodies).

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use crate::UiLang;
use crate::capabilities::Capabilities;
use crate::plan::VmSpoofProfile;
use crate::state::VmPowerState;

/// Center → host control request.
///
/// **Wire stability**: new variants append at the end; bump [`crate::PROTOCOL_VERSION`] when
/// breaking layout is unavoidable. rkyv discriminant follows declaration order (`Ping` = 0).
///
/// Guest memory / mouse uses the separate JSON guest-agent TCP protocol. VM → agent addresses on
/// the host are configured out-of-band (for example `agent-bindings.toml`); there is no
/// control-plane registration request in this protocol version.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub enum ControlRequest {
    /// Liveness + capability snapshot (same payload shape as [`ControlRequest::Hello`] response).
    Ping,
    /// Session handshake; host replies with [`ControlResponse::HelloAck`].
    Hello,
    /// Enumerate VMs on the host (OpenVMM-backed integration when wired; empty or stub otherwise).
    ListVms,
    /// Start each named VM (best-effort; see [`ControlResponse::BatchPowerAck`]).
    StartVmGroup { vm_names: Vec<String> },
    /// Stop each named VM (best-effort).
    StopVmGroup { vm_names: Vec<String> },
    /// Record script artifact metadata for a later load path (no large body on wire).
    SetScriptArtifact { version: String, sha256_hex: String },
    /// Load or replace a per-VM Lua chunk and execute it once (bounded by host policy).
    LoadScriptVm { vm_name: String, source: String },
    /// Apply host-side [`VmSpoofProfile`] steps to an existing VM (host automation; Windows when wired).
    ApplySpoofProfile {
        vm_name: String,
        dry_run: bool,
        spoof: VmSpoofProfile,
    },
    /// Apply a single spoof step by id (host implementation may be absent in slim builds).
    ApplySpoofStep {
        vm_name: String,
        step_id: String,
        dry_run: bool,
    },
    /// Capture the host OS primary display, downscale, and return JPEG bytes.
    HostDesktopSnapshot {
        /// Longer edge cap (pixels); host scales down preserving aspect.
        max_width: u32,
        max_height: u32,
        /// JPEG quality 1–100.
        jpeg_quality: u8,
    },
    /// One-shot host machine CPU / memory / network throughput snapshot.
    HostResourceSnapshot,
    /// Replace host UI / serve binding JSON (same JSON key as `titan_host_ui_v1` in Titan Host persistence).
    ApplyHostUiPersistJson { json: String },
    /// Center asks the host UI to switch display language (egui thread applies on next frame).
    SetUiLang { lang: UiLang },
    /// Center pushes the authoritative `VmWindowRecord` rows that belong to this host
    /// (`device_id` matches the host's own id). Host replaces its in-memory list and the
    /// `panel_window_mgmt` viewer redraws on the next frame.
    ///
    /// `records_json` is the JSON-encoded `Vec<VmWindowRecord>` already filtered for the recipient
    /// host. Center is the sole source of truth (Center-side SQLite); host stores nothing on disk.
    ApplyVmWindowSnapshot {
        device_id: String,
        records_json: String,
    },
}

/// One row in a [`ControlResponse::VmList`] payload.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub struct VmBrief {
    pub name: String,
    pub state: VmPowerState,
}

/// Host → center response.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
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
    /// Answer to [`ControlRequest::HostDesktopSnapshot`].
    DesktopSnapshotJpeg {
        jpeg_bytes: Vec<u8>,
        width_px: u32,
        height_px: u32,
    },
    /// Answer to [`ControlRequest::HostResourceSnapshot`].
    HostResourceSnapshot {
        stats: HostResourceStats,
    },
    /// Result of [`ControlRequest::ApplyHostUiPersistJson`].
    HostUiPersistAck {
        ok: bool,
        detail: String,
    },
    /// Result of [`ControlRequest::SetUiLang`].
    SetUiLangAck {
        ok: bool,
    },
    /// Result of [`ControlRequest::ApplyVmWindowSnapshot`]. `applied` is the row count adopted
    /// by the host (rejected requests with mismatched `device_id` set `ok = false`).
    ApplyVmWindowSnapshotAck {
        ok: bool,
        applied: u32,
        #[serde(default)]
        detail: String,
    },
}

/// Host machine resource snapshot (CPU / RAM / NIC totals; rates from host-side deltas).
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub struct HostResourceStats {
    /// System-wide CPU usage in permille (0–1000 ≈ 0–100%).
    pub cpu_permille: u32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    /// Receive bytes per second (download).
    pub net_down_bps: u64,
    /// Transmit bytes per second (upload).
    pub net_up_bps: u64,
    /// Aggregate disk read bytes per second (sum of mounted volumes’ counters).
    pub disk_read_bps: u64,
    /// Aggregate disk write bytes per second.
    pub disk_write_bps: u64,
}

/// One mounted volume / filesystem for telemetry.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub struct DiskVolume {
    pub mount: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

/// Host → center push payload (telemetry TCP; event-driven).
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub enum ControlPush {
    HostTelemetry {
        vms: Vec<VmBrief>,
        volumes: Vec<DiskVolume>,
        #[serde(default)]
        content_hint: Option<String>,
    },
    /// Periodic CPU / memory / NIC rates (telemetry TCP only; sent while at least one subscriber is connected).
    HostResourceLive { stats: HostResourceStats },
    /// Live host primary display preview (telemetry TCP only; JPEG; sent while subscribers connected).
    HostDesktopPreviewJpeg {
        jpeg_bytes: Vec<u8>,
        width_px: u32,
        height_px: u32,
    },
}

/// Center → host framed request (command TCP).
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub struct ControlRequestFrame {
    pub id: u64,
    pub body: ControlRequest,
}

/// Host → center frame on command TCP (response correlates to [`ControlRequestFrame::id`]).
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub enum ControlHostFrame {
    Response {
        id: u64,
        body: ControlResponse,
    },
    /// Optional on command socket; primary path is dedicated telemetry TCP.
    Push(ControlPush),
}
