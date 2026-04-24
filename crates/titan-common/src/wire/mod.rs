//! Framed binary control plane between center and host (TCP + rkyv).
//!
//! Layout: `MAGIC` (8) + `protocol_version` (u32 LE) + `payload_len` (u32 LE) + `payload` (rkyv).

mod types;

pub mod codec;
pub mod compress;
pub mod fleet_rkyv;
pub mod frame_bytes;

pub use codec::{
    control_plane_quic_addr, control_plane_telemetry_addr, decode_control_host_payload,
    decode_control_request_payload, decode_request_payload, decode_response_payload,
    decode_telemetry_push_payload, encode_control_host_frame, encode_control_request_frame,
    encode_request_frame, encode_response_frame, encode_telemetry_push_frame, parse_header,
    read_control_host_frame, read_control_request_frame, read_response_frame,
    read_telemetry_push_frame, telemetry_push_payload_fits, write_raw_frame, WireError, WireResult,
    CONTROL_PLANE_QUIC_PORT_OFFSET, CONTROL_PLANE_TELEMETRY_PORT_OFFSET, FRAME_HEADER_LEN,
    MAX_PAYLOAD_BYTES, TELEMETRY_MAX_PAYLOAD_BYTES, WIRE_MAGIC,
};
pub use types::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
    DiskVolume, HostResourceStats, VmBrief,
};

#[cfg(test)]
mod tests;
