//! Shared types and placeholder interfaces for Titan-v (center + host).

#![forbid(unsafe_code)]

pub mod capabilities;
pub mod discovery;
pub mod error;
pub mod need_mapping;
pub mod plan;
pub mod proxy_pool;
pub mod state;
pub mod stubs;
pub mod transport;
pub mod ui_lang;
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
pub use proxy_pool::{ProxyPoolEntry, VmProxyBinding};
pub use state::{NodeState, VmPowerState};
pub use stubs::{
    GpuPartitioner, HardwareSpoofer, NoopGpuPartitioner, NoopHardwareSpoofer, NoopStreamEncoder,
    NoopVmbusInput, StreamEncoder, VmbusInput,
};
pub use transport::{
    DdsControlBus, GrpcControlPlane, NoopDdsControlBus, NoopGrpcControlPlane, TcpWirePingClient,
    TracingHeartbeatBus,
};
pub use ui_lang::UiLang;
pub use wire::compress::{maybe_zstd_compress, zstd_compress_all, zstd_decompress_all};
pub use wire::fleet_rkyv::{FleetRkyvPing, fleet_rkyv_decode_ping, fleet_rkyv_encode_ping};
pub use wire::frame_bytes::{skip_bytes, take_payload_bytes};
pub use wire::{
    CONTROL_PLANE_QUIC_PORT_OFFSET, CONTROL_PLANE_TELEMETRY_PORT_OFFSET, ControlHostFrame,
    ControlPush, ControlRequest, ControlRequestFrame, ControlResponse, DiskVolume,
    FRAME_HEADER_LEN, HostResourceStats, MAX_PAYLOAD_BYTES, TELEMETRY_MAX_PAYLOAD_BYTES, VmBrief,
    WIRE_MAGIC, WireError, control_plane_quic_addr, control_plane_telemetry_addr,
    decode_control_host_payload, decode_control_request_payload, decode_request_payload,
    decode_response_payload, decode_telemetry_push_payload, encode_control_host_frame,
    encode_control_request_frame, encode_request_frame, encode_response_frame,
    encode_telemetry_push_frame, parse_header, read_control_host_frame, read_control_request_frame,
    read_response_frame, telemetry_push_payload_fits, write_raw_frame,
};

/// Wire protocol / capability negotiation version (center ↔ host).
pub const PROTOCOL_VERSION: u32 = 14;
