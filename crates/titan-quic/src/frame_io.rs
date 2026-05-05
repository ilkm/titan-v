//! `MAGIC + version + len + payload` frame I/O over QUIC streams.
//!
//! We reuse the existing wire framing from `titan-common` so the rkyv payload format and the
//! `PROTOCOL_VERSION` gate are unchanged from the legacy TCP transport. QUIC streams are
//! reliable byte streams, so each frame still gets its own length prefix to keep the readers
//! self-synchronising on partial reads / cancellations.

use anyhow::{Context, Result, anyhow};
use quinn::{RecvStream, SendStream};
use titan_common::{
    ControlHostFrame, ControlPush, ControlRequestFrame, FRAME_HEADER_LEN, MAX_PAYLOAD_BYTES,
    TELEMETRY_MAX_PAYLOAD_BYTES, decode_control_host_payload, decode_control_request_payload,
    decode_telemetry_push_payload, encode_control_host_frame, encode_control_request_frame,
    encode_telemetry_push_frame, parse_header,
};

/// Writes one rkyv-encoded `ControlRequestFrame` to a bidirectional QUIC stream.
pub async fn write_control_request(
    send: &mut SendStream,
    frame: &ControlRequestFrame,
) -> Result<()> {
    let bytes = encode_control_request_frame(frame).context("encode control request")?;
    send.write_all(&bytes)
        .await
        .context("quic send write_all")?;
    Ok(())
}

/// Reads one full `ControlRequestFrame` from `recv`. Returns `Ok(None)` on clean EOF.
pub async fn read_one_control_request(
    recv: &mut RecvStream,
) -> Result<Option<ControlRequestFrame>> {
    let Some(payload) = read_one_payload(recv, MAX_PAYLOAD_BYTES).await? else {
        return Ok(None);
    };
    Ok(Some(
        decode_control_request_payload(&payload).context("decode control request")?,
    ))
}

/// Writes one rkyv-encoded `ControlHostFrame` (response or push embedded in control plane).
pub async fn write_control_host(send: &mut SendStream, frame: &ControlHostFrame) -> Result<()> {
    let bytes = encode_control_host_frame(frame).context("encode control host frame")?;
    send.write_all(&bytes)
        .await
        .context("quic send write_all")?;
    Ok(())
}

/// Reads one `ControlHostFrame` from `recv`. Returns `Ok(None)` on clean EOF.
pub async fn read_one_control_host(recv: &mut RecvStream) -> Result<Option<ControlHostFrame>> {
    let Some(payload) = read_one_payload(recv, MAX_PAYLOAD_BYTES).await? else {
        return Ok(None);
    };
    Ok(Some(
        decode_control_host_payload(&payload).context("decode control host frame")?,
    ))
}

/// Writes one telemetry `ControlPush` onto a unidirectional Host→Center stream.
pub async fn write_telemetry_push(send: &mut SendStream, push: &ControlPush) -> Result<()> {
    let bytes = encode_telemetry_push_frame(push).context("encode telemetry push")?;
    send.write_all(&bytes)
        .await
        .context("quic telemetry write_all")?;
    Ok(())
}

/// Reads one `ControlPush` from a unidirectional telemetry stream. `Ok(None)` on clean EOF.
pub async fn read_one_telemetry_push(recv: &mut RecvStream) -> Result<Option<ControlPush>> {
    let Some(payload) = read_one_payload(recv, TELEMETRY_MAX_PAYLOAD_BYTES).await? else {
        return Ok(None);
    };
    Ok(Some(
        decode_telemetry_push_payload(&payload).context("decode telemetry push")?,
    ))
}

async fn read_one_payload(recv: &mut RecvStream, max: u32) -> Result<Option<Vec<u8>>> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    if !read_exact_or_eof(recv, &mut hdr).await? {
        return Ok(None);
    }
    let (_ver, len) = parse_header(&hdr).context("parse frame header")?;
    if len > max {
        return Err(anyhow!("payload {len} exceeds max {max}"));
    }
    let mut payload = vec![0u8; len as usize];
    if !read_exact_or_eof(recv, &mut payload).await? {
        return Err(anyhow!("unexpected eof inside frame body"));
    }
    Ok(Some(payload))
}

async fn read_exact_or_eof(recv: &mut RecvStream, buf: &mut [u8]) -> Result<bool> {
    let mut off = 0usize;
    while off < buf.len() {
        match recv.read(&mut buf[off..]).await {
            Ok(Some(0)) => return Ok(false),
            Ok(Some(n)) => off += n,
            Ok(None) => {
                if off == 0 {
                    return Ok(false);
                }
                return Err(anyhow!("eof inside frame after {off} bytes"));
            }
            Err(e) => return Err(anyhow!("quic recv: {e}")),
        }
    }
    Ok(true)
}
