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
    DiscoveryBeacon, DEFAULT_DISCOVERY_UDP_PORT, DISCOVERY_BEACON_KIND, DISCOVERY_SCHEMA_VERSION,
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
pub use wire::{
    decode_request_payload, decode_response_payload, encode_request_frame, encode_response_frame,
    parse_header, read_request_frame, read_response_frame, write_raw_frame, ControlRequest,
    ControlResponse, VmBrief, WireError, FRAME_HEADER_LEN, MAX_PAYLOAD_BYTES, WIRE_MAGIC,
};

/// Wire protocol / capability negotiation version (center ↔ host).
pub const PROTOCOL_VERSION: u32 = 4;
