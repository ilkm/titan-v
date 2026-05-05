//! Framed binary codec: `MAGIC` + version + length + rkyv payload.
//!
//! The transport (QUIC stream, in-memory channel, etc.) is owned by the caller.
//! This module only does encode/decode and header validation.

use bytes::BytesMut;
use thiserror::Error;

use super::types::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
};
use crate::PROTOCOL_VERSION;

/// Wire magic `TITANV01` (8 bytes, ASCII). The trailing version digits are decorative; the
/// authoritative protocol version is the `u32` that follows in [`FRAME_HEADER_LEN`].
pub const WIRE_MAGIC: [u8; 8] = *b"TITANV01";

/// Maximum payload bytes after header (defense in depth).
pub const MAX_PAYLOAD_BYTES: u32 = 512 * 1024;

/// Cap for telemetry-only frames (VM list + disk + optional JPEG desktop preview push).
pub const TELEMETRY_MAX_PAYLOAD_BYTES: u32 = 192 * 1024;

/// Byte length of the wire header (`MAGIC` + version + payload length).
pub const FRAME_HEADER_LEN: usize = 8 + 4 + 4;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("invalid frame magic")]
    BadMagic,
    #[error("unsupported protocol version {got} (expected {expected})")]
    UnsupportedVersion { got: u32, expected: u32 },
    #[error("payload length {0} exceeds maximum {MAX_PAYLOAD_BYTES}")]
    PayloadTooLarge(u32),
    #[error("unexpected end of frame")]
    UnexpectedEof,
    #[error("rkyv decode: {0}")]
    Decode(String),
    #[error("rkyv encode: {0}")]
    Encode(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}

pub type WireResult<T> = std::result::Result<T, WireError>;

fn push_wire_header(buf: &mut BytesMut, payload_len: u32) {
    buf.reserve(FRAME_HEADER_LEN + payload_len as usize);
    buf.extend_from_slice(&WIRE_MAGIC);
    buf.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    buf.extend_from_slice(&payload_len.to_le_bytes());
}

fn wrap_aligned_payload(aligned: rkyv::util::AlignedVec, max_payload: u32) -> WireResult<Vec<u8>> {
    let len: u32 = aligned
        .len()
        .try_into()
        .map_err(|_| WireError::PayloadTooLarge(u32::MAX))?;
    if len > max_payload {
        return Err(WireError::PayloadTooLarge(len));
    }
    let mut buf = BytesMut::with_capacity(FRAME_HEADER_LEN + aligned.len());
    push_wire_header(&mut buf, len);
    buf.extend_from_slice(aligned.as_slice());
    Ok(buf.freeze().to_vec())
}

pub fn encode_control_request_frame(frame: &ControlRequestFrame) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(frame)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

pub fn encode_control_host_frame(frame: &ControlHostFrame) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(frame)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

#[must_use]
pub fn telemetry_push_payload_fits(push: &ControlPush) -> bool {
    match rkyv::to_bytes::<rkyv::rancor::Error>(push) {
        Ok(v) => v.len() as u32 <= TELEMETRY_MAX_PAYLOAD_BYTES,
        Err(_) => false,
    }
}

pub fn encode_telemetry_push_frame(push: &ControlPush) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(push)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, TELEMETRY_MAX_PAYLOAD_BYTES)
}

/// Encodes a full frame for `req`; uses `id == 1` (testing / one-shot helpers).
pub fn encode_request_frame(req: &ControlRequest) -> WireResult<Vec<u8>> {
    encode_control_request_frame(&ControlRequestFrame {
        id: 1,
        body: req.clone(),
    })
}

/// Encodes a raw `ControlResponse` body (used by tests).
pub fn encode_response_frame(res: &ControlResponse) -> WireResult<Vec<u8>> {
    let aligned =
        rkyv::to_bytes::<rkyv::rancor::Error>(res).map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

pub fn parse_header(header: &[u8; FRAME_HEADER_LEN]) -> WireResult<(u32, u32)> {
    if header[0..8] != WIRE_MAGIC {
        return Err(WireError::BadMagic);
    }
    let ver = u32::from_le_bytes(
        header[8..12]
            .try_into()
            .map_err(|_| WireError::UnexpectedEof)?,
    );
    if ver != PROTOCOL_VERSION {
        return Err(WireError::UnsupportedVersion {
            got: ver,
            expected: PROTOCOL_VERSION,
        });
    }
    let len = u32::from_le_bytes(
        header[12..16]
            .try_into()
            .map_err(|_| WireError::UnexpectedEof)?,
    );
    if len > MAX_PAYLOAD_BYTES {
        return Err(WireError::PayloadTooLarge(len));
    }
    Ok((ver, len))
}

pub fn decode_control_request_payload(payload: &[u8]) -> WireResult<ControlRequestFrame> {
    rkyv::from_bytes::<ControlRequestFrame, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

pub fn decode_control_host_payload(payload: &[u8]) -> WireResult<ControlHostFrame> {
    rkyv::from_bytes::<ControlHostFrame, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

pub fn decode_telemetry_push_payload(payload: &[u8]) -> WireResult<ControlPush> {
    rkyv::from_bytes::<ControlPush, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

pub fn decode_response_payload(payload: &[u8]) -> WireResult<ControlResponse> {
    rkyv::from_bytes::<ControlResponse, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

#[cfg(test)]
pub(crate) fn read_control_request_frame<R: std::io::Read>(
    r: &mut R,
) -> WireResult<ControlRequestFrame> {
    let payload = read_payload_blocking(r, MAX_PAYLOAD_BYTES)?;
    decode_control_request_payload(&payload)
}

#[cfg(test)]
pub(crate) fn read_control_host_frame<R: std::io::Read>(r: &mut R) -> WireResult<ControlHostFrame> {
    let payload = read_payload_blocking(r, MAX_PAYLOAD_BYTES)?;
    decode_control_host_payload(&payload)
}

#[cfg(test)]
pub(crate) fn read_response_frame<R: std::io::Read>(r: &mut R) -> WireResult<ControlResponse> {
    let payload = read_payload_blocking(r, MAX_PAYLOAD_BYTES)?;
    decode_response_payload(&payload)
}

#[cfg(test)]
pub(crate) fn read_telemetry_push_frame<R: std::io::Read>(r: &mut R) -> WireResult<ControlPush> {
    let payload = read_payload_blocking(r, TELEMETRY_MAX_PAYLOAD_BYTES)?;
    decode_telemetry_push_payload(&payload)
}

#[cfg(test)]
fn read_payload_blocking<R: std::io::Read>(r: &mut R, max: u32) -> WireResult<Vec<u8>> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    r.read_exact(&mut hdr)?;
    let (_ver, len) = parse_header(&hdr)?;
    if len > max {
        return Err(WireError::PayloadTooLarge(len));
    }
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok(payload)
}
