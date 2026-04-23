//! Framed binary control plane between center and host (M2).
//!
//! Layout: `MAGIC` (8) + `protocol_version` (u32 LE) + `payload_len` (u32 LE) + `payload` (postcard).

mod types;

pub mod codec;

pub use codec::{
    decode_request_payload, decode_response_payload, encode_request_frame, encode_response_frame,
    parse_header, read_request_frame, read_response_frame, write_raw_frame, WireError, WireResult,
    FRAME_HEADER_LEN, MAX_PAYLOAD_BYTES, WIRE_MAGIC,
};
pub use types::{ControlRequest, ControlResponse, VmBrief};

#[cfg(test)]
mod tests;
