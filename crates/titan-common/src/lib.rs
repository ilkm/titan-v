//! Shared types and placeholder interfaces for Titan-v (center + host).

#![forbid(unsafe_code)]

pub mod capabilities;
pub mod discovery;
pub mod error;
pub mod plan;
pub mod state;
pub mod ui_lang;
pub mod vm_window;
pub mod wire;

pub use capabilities::{Capabilities, HostRuntimeProbes, HostSpoofProbeCaps};
pub use discovery::{
    CENTER_POLL_BEACON_KIND, CENTER_POLL_SCHEMA_VERSION, CenterPollBeacon,
    DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT, DEFAULT_DISCOVERY_UDP_PORT,
    DISCOVERY_BEACON_KIND, DISCOVERY_SCHEMA_VERSION, DiscoveryBeacon, HOST_ANNOUNCE_BEACON_KIND,
    HOST_ANNOUNCE_SCHEMA_VERSION, HostAnnounceBeacon,
};
pub use error::{Error, Result};
pub use plan::VmSpoofProfile;
pub use state::{NodeState, VmPowerState};
pub use ui_lang::UiLang;
pub use vm_window::{
    VM_WINDOW_FOLDER_ID_MAX, VM_WINDOW_FOLDER_ID_MIN, VmWindowRecord, next_unused_vm_folder_id,
    validate_vm_window_record,
};
pub use wire::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
    DiskVolume, FRAME_HEADER_LEN, HostResourceStats, MAX_PAYLOAD_BYTES,
    TELEMETRY_MAX_PAYLOAD_BYTES, VmBrief, WIRE_MAGIC, WireError, decode_control_host_payload,
    decode_control_request_payload, decode_response_payload, decode_telemetry_push_payload,
    encode_control_host_frame, encode_control_request_frame, encode_request_frame,
    encode_response_frame, encode_telemetry_push_frame, parse_header, telemetry_push_payload_fits,
};

/// Wire protocol / capability negotiation version (center ↔ host).
///
/// `16`: full TCP→QUIC+mTLS transport replacement (`titan-quic` crate, schema v3 host announce
/// with SPKI fingerprint, `SubscribeTelemetry` RPC + uni-stream telemetry pump, ALPN
/// `titan-control-v1` / `titan-telemetry-v1`). No backward compatibility with v1/v2 frames.
pub const PROTOCOL_VERSION: u32 = 16;
