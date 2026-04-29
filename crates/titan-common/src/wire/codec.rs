//! Framed binary codec: `MAGIC` + version + length + rkyv payload.

use std::io::Read;

use bytes::BytesMut;
use thiserror::Error;

use std::net::SocketAddr;

use super::types::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
};
use crate::PROTOCOL_VERSION;

/// Wire magic `TITANV01` (8 bytes, ASCII).
pub const WIRE_MAGIC: [u8; 8] = *b"TITANV01";

/// Maximum payload bytes after header (defense in depth).
/// Large enough for a downscaled desktop JPEG (see [`ControlRequest::HostDesktopSnapshot`]).
pub const MAX_PAYLOAD_BYTES: u32 = 512 * 1024;

/// Cap for telemetry-only frames (VM list + disk + optional JPEG desktop preview push).
pub const TELEMETRY_MAX_PAYLOAD_BYTES: u32 = 192 * 1024;

/// TCP port offset from control-plane command listen port → telemetry listen port (same IP; TCP vs UDP may share numeric port).
pub const CONTROL_PLANE_TELEMETRY_PORT_OFFSET: u16 = 1;

/// Telemetry TCP address paired with a control-plane command listen `addr` (`ip:port` + offset).
#[must_use]
pub fn control_plane_telemetry_addr(command: SocketAddr) -> SocketAddr {
    SocketAddr::new(
        command.ip(),
        command
            .port()
            .saturating_add(CONTROL_PLANE_TELEMETRY_PORT_OFFSET),
    )
}

/// Byte length of the wire header (`MAGIC` + version + payload length).
pub const FRAME_HEADER_LEN: usize = 8 + 4 + 4;

/// Wire codec errors (no secrets in `Display`).
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

/// Encodes a full control-plane request frame (center → host on command TCP).
pub fn encode_control_request_frame(frame: &ControlRequestFrame) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(frame)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

/// Encodes a full control-plane host frame (response or push on command TCP).
pub fn encode_control_host_frame(frame: &ControlHostFrame) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(frame)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

/// Whether `push` serializes to a telemetry payload within [`TELEMETRY_MAX_PAYLOAD_BYTES`].
///
/// Cheaper than [`encode_telemetry_push_frame`]: one rkyv buffer, no wire header allocation.
#[must_use]
pub fn telemetry_push_payload_fits(push: &ControlPush) -> bool {
    match rkyv::to_bytes::<rkyv::rancor::Error>(push) {
        Ok(v) => v.len() as u32 <= TELEMETRY_MAX_PAYLOAD_BYTES,
        Err(_) => false,
    }
}

/// Encodes a telemetry push (telemetry TCP only).
pub fn encode_telemetry_push_frame(push: &ControlPush) -> WireResult<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(push)
        .map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, TELEMETRY_MAX_PAYLOAD_BYTES)
}

/// Encodes a full frame (header + rkyv payload) for `req`.
///
/// Uses [`ControlRequestFrame`] with `id == 1` for tests and legacy call sites.
pub fn encode_request_frame(req: &ControlRequest) -> WireResult<Vec<u8>> {
    encode_control_request_frame(&ControlRequestFrame {
        id: 1,
        body: req.clone(),
    })
}

/// Encodes a full response frame (legacy: raw [`ControlResponse`] without framed host envelope).
pub fn encode_response_frame(res: &ControlResponse) -> WireResult<Vec<u8>> {
    let aligned =
        rkyv::to_bytes::<rkyv::rancor::Error>(res).map_err(|e| WireError::Encode(e.to_string()))?;
    wrap_aligned_payload(aligned, MAX_PAYLOAD_BYTES)
}

/// Parses the fixed wire header; returns `(protocol_version, payload_len)`.
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

/// Decodes control-plane request body (rkyv only).
pub fn decode_control_request_payload(payload: &[u8]) -> WireResult<ControlRequestFrame> {
    rkyv::from_bytes::<ControlRequestFrame, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

/// Decodes control-plane host frame body (rkyv only).
pub fn decode_control_host_payload(payload: &[u8]) -> WireResult<ControlHostFrame> {
    rkyv::from_bytes::<ControlHostFrame, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

/// Decodes telemetry push body (rkyv only).
pub fn decode_telemetry_push_payload(payload: &[u8]) -> WireResult<ControlPush> {
    rkyv::from_bytes::<ControlPush, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

/// Decodes a response payload (rkyv body only).
pub fn decode_response_payload(payload: &[u8]) -> WireResult<ControlResponse> {
    rkyv::from_bytes::<ControlResponse, rkyv::rancor::Error>(payload)
        .map_err(|e| WireError::Decode(e.to_string()))
}

/// Reads one control-plane request frame from `r`.
pub fn read_control_request_frame<R: Read>(r: &mut R) -> WireResult<ControlRequestFrame> {
    let (payload, _) = read_payload(r)?;
    decode_control_request_payload(&payload)
}

/// Reads one control-plane host frame from `r`.
pub fn read_control_host_frame<R: Read>(r: &mut R) -> WireResult<ControlHostFrame> {
    let (payload, _) = read_payload(r)?;
    decode_control_host_payload(&payload)
}

/// Reads one frame from `r`, returning the decoded [`ControlResponse`] (legacy raw body).
pub fn read_response_frame<R: Read>(r: &mut R) -> WireResult<ControlResponse> {
    let (payload, _) = read_payload(r)?;
    decode_response_payload(&payload)
}

/// Reads one telemetry push from `r` (telemetry TCP).
pub fn read_telemetry_push_frame<R: Read>(r: &mut R) -> WireResult<ControlPush> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    r.read_exact(&mut hdr)?;
    let (_ver, len) = parse_header(&hdr)?;
    if len > TELEMETRY_MAX_PAYLOAD_BYTES {
        return Err(WireError::PayloadTooLarge(len));
    }
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    decode_telemetry_push_payload(&payload)
}

fn read_payload<R: Read>(r: &mut R) -> WireResult<(Vec<u8>, u32)> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    r.read_exact(&mut hdr)?;
    let (ver, len) = parse_header(&hdr)?;
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok((payload, ver))
}
