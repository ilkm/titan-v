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
pub mod wire;

pub use capabilities::{Capabilities, HostRuntimeProbes, HypervSpoofHostCaps};
pub use discovery::{
    CenterPollBeacon, DiscoveryBeacon, HostAnnounceBeacon, CENTER_POLL_BEACON_KIND,
    CENTER_POLL_SCHEMA_VERSION, DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT,
    DEFAULT_DISCOVERY_UDP_PORT, DISCOVERY_BEACON_KIND, DISCOVERY_SCHEMA_VERSION,
    HOST_ANNOUNCE_BEACON_KIND, HOST_ANNOUNCE_SCHEMA_VERSION,
};
pub use error::{Error, Result};
pub use plan::{VmIdentityProfile, VmProvisionPlan, VmSpoofProfile, PLAN_FORMAT_VERSION};
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
pub use wire::compress::{maybe_zstd_compress, zstd_compress_all, zstd_decompress_all};
pub use wire::fleet_rkyv::{fleet_rkyv_decode_ping, fleet_rkyv_encode_ping, FleetRkyvPing};
pub use wire::frame_bytes::{skip_bytes, take_payload_bytes};
pub use wire::{
    control_plane_quic_addr, control_plane_telemetry_addr, decode_control_host_payload,
    decode_control_request_payload, decode_request_payload, decode_response_payload,
    decode_telemetry_push_payload, encode_control_host_frame, encode_control_request_frame,
    encode_request_frame, encode_response_frame, encode_telemetry_push_frame, parse_header,
    read_control_host_frame, read_control_request_frame, read_response_frame,
    telemetry_push_payload_fits, write_raw_frame, ControlHostFrame, ControlPush, ControlRequest,
    ControlRequestFrame, ControlResponse, DiskVolume, HostResourceStats, VmBrief, WireError,
    CONTROL_PLANE_QUIC_PORT_OFFSET, CONTROL_PLANE_TELEMETRY_PORT_OFFSET, FRAME_HEADER_LEN,
    MAX_PAYLOAD_BYTES, TELEMETRY_MAX_PAYLOAD_BYTES, WIRE_MAGIC,
};

/// Wire protocol / capability negotiation version (center ↔ host).
pub const PROTOCOL_VERSION: u32 = 13;
