//! Framed binary codec: `MAGIC` + version + length + postcard payload.

use std::io::{Read, Write};

use thiserror::Error;

use super::types::{ControlRequest, ControlResponse};
use crate::PROTOCOL_VERSION;

/// Wire magic `TITANV01` (8 bytes, ASCII).
pub const WIRE_MAGIC: [u8; 8] = *b"TITANV01";

/// Maximum payload bytes after header (defense in depth).
pub const MAX_PAYLOAD_BYTES: u32 = 64 * 1024;

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
    #[error("postcard decode: {0}")]
    Decode(String),
    #[error("postcard encode: {0}")]
    Encode(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}

pub type WireResult<T> = std::result::Result<T, WireError>;

fn encode_postcard_payload(payload: Vec<u8>) -> WireResult<Vec<u8>> {
    let len: u32 = payload
        .len()
        .try_into()
        .map_err(|_| WireError::PayloadTooLarge(u32::MAX))?;
    if len > MAX_PAYLOAD_BYTES {
        return Err(WireError::PayloadTooLarge(len));
    }
    let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    out.extend_from_slice(&WIRE_MAGIC);
    out.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Encodes a full frame (header + postcard payload) for `req`.
pub fn encode_request_frame(req: &ControlRequest) -> WireResult<Vec<u8>> {
    let payload = postcard::to_allocvec(req).map_err(|e| WireError::Encode(e.to_string()))?;
    encode_postcard_payload(payload)
}

/// Encodes a full response frame.
pub fn encode_response_frame(res: &ControlResponse) -> WireResult<Vec<u8>> {
    let payload = postcard::to_allocvec(res).map_err(|e| WireError::Encode(e.to_string()))?;
    encode_postcard_payload(payload)
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

/// Decodes a request payload (postcard body only).
pub fn decode_request_payload(payload: &[u8]) -> WireResult<ControlRequest> {
    postcard::from_bytes(payload).map_err(|e| WireError::Decode(e.to_string()))
}

/// Decodes a response payload (postcard body only).
pub fn decode_response_payload(payload: &[u8]) -> WireResult<ControlResponse> {
    postcard::from_bytes(payload).map_err(|e| WireError::Decode(e.to_string()))
}

/// Reads one frame from `r`, returning the decoded [`ControlRequest`].
pub fn read_request_frame<R: Read>(r: &mut R) -> WireResult<ControlRequest> {
    let (payload, _) = read_payload(r)?;
    decode_request_payload(&payload)
}

/// Reads one frame from `r`, returning the decoded [`ControlResponse`].
pub fn read_response_frame<R: Read>(r: &mut R) -> WireResult<ControlResponse> {
    let (payload, _) = read_payload(r)?;
    decode_response_payload(&payload)
}

fn read_payload<R: Read>(r: &mut R) -> WireResult<(Vec<u8>, u32)> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    r.read_exact(&mut hdr)?;
    let (ver, len) = parse_header(&hdr)?;
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok((payload, ver))
}

/// Writes a pre-built full frame (from [`encode_request_frame`] / [`encode_response_frame`]).
pub fn write_raw_frame<W: Write>(w: &mut W, frame: &[u8]) -> WireResult<()> {
    w.write_all(frame)?;
    w.flush()?;
    Ok(())
}
