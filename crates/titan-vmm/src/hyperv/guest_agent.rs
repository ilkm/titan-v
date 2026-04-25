//! Guest Agent TCP protocol (cooperative guest path; **not** paravisor / WinHv).
//!
//! # Security and compliance
//!
//! - No authentication in v1: **bind to loopback or trusted networks only**. Do not expose the
//!   agent port on untrusted interfaces without TLS + PSK (future work).
//! - Reading guest memory or injecting input may violate **game EULA** or local law; operators
//!   are responsible for lawful use.
//! - Logs must not include full script bodies or credentials ([`titan_common`] host rules).
//!
//! # Framing
//!
//! Each message: `u32` **big-endian** length of UTF-8 JSON payload, followed by payload bytes.
//! Maximum frame size: [`MAX_AGENT_FRAME`] (64 KiB).
//!
//! # Request JSON (v1)
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `v` | number | Protocol version, must be `1` |
//! | `op` | string | `read_u64` \| `mouse_move` \| `ping` \| `stream_capabilities` \| `identity_echo` \| `identity_ops` |
//! | `id` | string | Correlation id for logs |
//! | `vm_id` | string | Expected Hyper-V VM name; agent may reject mismatch |
//! | `payload` | object | Operation-specific |
//!
//! ## `read_u64` payload
//!
//! - `address` (string): decimal or `0x` hex guest virtual address (interpreted **inside** the agent).
//! - `pid` (optional number): target process id if the agent supports scoped reads.
//!
//! ## `mouse_move` payload
//!
//! - `x`, `y` (numbers): coordinates in guest space (agent-defined).
//!
//! # Response JSON
//!
//! - `ok` (bool)
//! - `id` (string): echoes request `id`
//! - `value` (string, optional): decimal `u64` string for `read_u64` success
//! - `error` (string, optional): short error message (no secrets)
//!
//! ## `identity_ops` payload (Phase 2A)
//!
//! Cooperative **IdentityOps** bundle for machine name / NIC refresh / artifact correlation (guest-defined keys).
//! - `action` (string): e.g. `echo` \| `refresh_nics` \| `artifact_ack` (extensible; unknown actions may error).
//! - `params` (object, optional): action-specific parameters (no secrets).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use titan_common::{Error, Result};

/// Maximum JSON body bytes for one agent frame (matches control-plane scale).
pub const MAX_AGENT_FRAME: u32 = 64 * 1024;

const PROTO_V1: u32 = 1;

#[derive(Debug, Serialize)]
struct AgentRequest<'a> {
    v: u32,
    op: &'a str,
    id: &'a str,
    vm_id: &'a str,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AgentResponse {
    ok: bool,
    id: String,
    #[serde(default)]
    value: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
}

fn agent_connect(addr: &SocketAddr, read_timeout: Duration) -> Result<TcpStream> {
    let stream =
        TcpStream::connect_timeout(addr, read_timeout).map_err(|e| Error::HyperVRejected {
            message: format!("guest agent connect {addr}: {e}"),
        })?;
    stream
        .set_read_timeout(Some(read_timeout))
        .map_err(Error::Io)?;
    stream
        .set_write_timeout(Some(read_timeout))
        .map_err(Error::Io)?;
    Ok(stream)
}

fn agent_encode_framed_request(
    vm_id: &str,
    op: &str,
    payload: serde_json::Value,
    request_id: &str,
) -> Result<Vec<u8>> {
    let req = AgentRequest {
        v: PROTO_V1,
        op,
        id: request_id,
        vm_id,
        payload,
    };
    let body = serde_json::to_vec(&req).map_err(|e| Error::HyperVRejected {
        message: format!("guest agent json encode: {e}"),
    })?;
    let len: u32 = body.len().try_into().map_err(|_| Error::HyperVRejected {
        message: "guest agent request too large".into(),
    })?;
    if len > MAX_AGENT_FRAME {
        return Err(Error::HyperVRejected {
            message: "guest agent request exceeds MAX_AGENT_FRAME".into(),
        });
    }
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&body);
    Ok(frame)
}

fn agent_read_framed_response(stream: &mut TcpStream, request_id: &str) -> Result<AgentResponse> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .map_err(|e| Error::HyperVRejected {
            message: format!("guest agent read length: {e}"),
        })?;
    let resp_len = u32::from_be_bytes(len_buf);
    if resp_len > MAX_AGENT_FRAME {
        return Err(Error::HyperVRejected {
            message: format!("guest agent response length {resp_len} too large"),
        });
    }
    let mut payload_buf = vec![0u8; resp_len as usize];
    stream
        .read_exact(&mut payload_buf)
        .map_err(|e| Error::HyperVRejected {
            message: format!("guest agent read body: {e}"),
        })?;
    let resp: AgentResponse =
        serde_json::from_slice(&payload_buf).map_err(|e| Error::HyperVRejected {
            message: format!("guest agent json decode: {e}"),
        })?;
    if resp.id != request_id {
        return Err(Error::HyperVRejected {
            message: "guest agent response id mismatch".into(),
        });
    }
    Ok(resp)
}

/// Sends one framed request and reads one framed response (blocking).
pub(crate) fn roundtrip(
    addr: &SocketAddr,
    vm_id: &str,
    op: &str,
    payload: serde_json::Value,
    request_id: &str,
    read_timeout: Duration,
) -> Result<AgentResponse> {
    let mut stream = agent_connect(addr, read_timeout)?;
    let frame = agent_encode_framed_request(vm_id, op, payload, request_id)?;
    stream.write_all(&frame).map_err(Error::Io)?;
    agent_read_framed_response(&mut stream, request_id)
}

fn guest_u64_coerce_value_text(v: serde_json::Value) -> Result<String> {
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(Error::HyperVRejected {
            message: "guest agent read_u64 value not string/number".into(),
        }),
    }
}

fn parse_u64_from_agent_text(s: &str) -> Result<u64> {
    let trimmed = s.trim();
    let n = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u64>()
    }
    .map_err(|e| Error::HyperVRejected {
        message: format!("guest agent read_u64 parse value: {e}"),
    })?;
    Ok(n)
}

/// Reads a `u64` from the guest agent at `addr` for VM `vm_id`.
pub fn read_guest_u64(
    addr: &SocketAddr,
    vm_id: &str,
    guest_addr: u64,
    request_id: &str,
    timeout: Duration,
) -> Result<u64> {
    let payload = serde_json::json!({
        "address": format!("0x{guest_addr:x}"),
    });
    let resp = roundtrip(addr, vm_id, "read_u64", payload, request_id, timeout)?;
    if !resp.ok {
        return Err(Error::HyperVRejected {
            message: resp
                .error
                .unwrap_or_else(|| "guest agent read_u64 failed".into()),
        });
    }
    let v = resp.value.ok_or_else(|| Error::HyperVRejected {
        message: "guest agent read_u64 missing value".into(),
    })?;
    let s = guest_u64_coerce_value_text(v)?;
    parse_u64_from_agent_text(&s)
}

/// Phase 2A hook: cooperative **identity / health** echo (guest must implement `op` = `identity_echo`).
///
/// Returns the JSON `value` field on success (typically includes echoed `vm_id` and a note).
pub fn identity_echo(
    addr: &SocketAddr,
    vm_id: &str,
    request_id: &str,
    timeout: Duration,
) -> Result<serde_json::Value> {
    let resp = roundtrip(
        addr,
        vm_id,
        "identity_echo",
        serde_json::json!({}),
        request_id,
        timeout,
    )?;
    if !resp.ok {
        return Err(Error::HyperVRejected {
            message: resp
                .error
                .unwrap_or_else(|| "guest agent identity_echo failed".into()),
        });
    }
    resp.value.ok_or_else(|| Error::HyperVRejected {
        message: "guest agent identity_echo missing value".into(),
    })
}

/// Sends **IdentityOps** (`identity_ops`) with caller-built JSON payload (`action`, optional `params`).
pub fn identity_ops(
    addr: &SocketAddr,
    vm_id: &str,
    payload: serde_json::Value,
    request_id: &str,
    timeout: Duration,
) -> Result<serde_json::Value> {
    let resp = roundtrip(addr, vm_id, "identity_ops", payload, request_id, timeout)?;
    if !resp.ok {
        return Err(Error::HyperVRejected {
            message: resp
                .error
                .unwrap_or_else(|| "guest agent identity_ops failed".into()),
        });
    }
    resp.value.ok_or_else(|| Error::HyperVRejected {
        message: "guest agent identity_ops missing value".into(),
    })
}

/// Injects a mouse move via the guest agent.
pub fn mouse_move(
    addr: &SocketAddr,
    vm_id: &str,
    x: u32,
    y: u32,
    request_id: &str,
    timeout: Duration,
) -> Result<()> {
    let payload = serde_json::json!({ "x": x, "y": y });
    let resp = roundtrip(addr, vm_id, "mouse_move", payload, request_id, timeout)?;
    if !resp.ok {
        return Err(Error::HyperVRejected {
            message: resp
                .error
                .unwrap_or_else(|| "guest agent mouse_move failed".into()),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn write_json_frame(w: &mut impl Write, body: &serde_json::Value) {
        let out = serde_json::to_vec(body).unwrap();
        let mut frame = Vec::new();
        frame.extend_from_slice(&(out.len() as u32).to_be_bytes());
        frame.extend_from_slice(&out);
        w.write_all(&frame).unwrap();
    }

    fn read_frame(mut r: impl Read) -> Vec<u8> {
        let mut h = [0u8; 4];
        r.read_exact(&mut h).unwrap();
        let n = u32::from_be_bytes(h) as usize;
        let mut b = vec![0u8; n];
        r.read_exact(&mut b).unwrap();
        b
    }

    #[test]
    fn mock_agent_read_u64_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let th = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let req_bytes = read_frame(&mut sock);
            let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
            assert_eq!(req["op"], "read_u64");
            let body = serde_json::json!({
                "ok": true,
                "id": req["id"],
                "value": "42"
            });
            let out = serde_json::to_vec(&body).unwrap();
            let mut frame = Vec::new();
            frame.extend_from_slice(&(out.len() as u32).to_be_bytes());
            frame.extend_from_slice(&out);
            sock.write_all(&frame).unwrap();
            let _ = tx.send(req_bytes);
        });
        let v = read_guest_u64(&addr, "vm-1", 0x1000, "t1", Duration::from_secs(2)).unwrap();
        assert_eq!(v, 42);
        th.join().unwrap();
        let got = rx.recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&got).unwrap();
        assert_eq!(parsed["vm_id"], "vm-1");
    }

    #[test]
    fn mock_agent_mouse_move() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let th = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let req_bytes = read_frame(&mut sock);
            let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
            assert_eq!(req["op"], "mouse_move");
            assert_eq!(req["payload"]["x"], 10);
            let body = serde_json::json!({ "ok": true, "id": req["id"] });
            let out = serde_json::to_vec(&body).unwrap();
            let mut frame = Vec::new();
            frame.extend_from_slice(&(out.len() as u32).to_be_bytes());
            frame.extend_from_slice(&out);
            sock.write_all(&frame).unwrap();
        });
        mouse_move(&addr, "vm-2", 10, 20, "m1", Duration::from_secs(2)).unwrap();
        th.join().unwrap();
    }

    #[test]
    fn mock_agent_identity_ops() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let th = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let req_bytes = read_frame(&mut sock);
            let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
            assert_eq!(req["op"], "identity_ops");
            assert_eq!(req["payload"]["action"], "echo");
            let body = serde_json::json!({
                "ok": true,
                "id": req["id"],
                "value": { "action": "echo", "vm_id": req["vm_id"] }
            });
            write_json_frame(&mut sock, &body);
        });
        let v = identity_ops(
            &addr,
            "vm-y",
            serde_json::json!({ "action": "echo" }),
            "io1",
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(v["vm_id"], "vm-y");
        th.join().unwrap();
    }

    #[test]
    fn mock_agent_identity_echo() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let th = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let req_bytes = read_frame(&mut sock);
            let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
            assert_eq!(req["op"], "identity_echo");
            let body = serde_json::json!({
                "ok": true,
                "id": req["id"],
                "value": { "vm_id": req["vm_id"], "phase": "2a" }
            });
            let out = serde_json::to_vec(&body).unwrap();
            let mut frame = Vec::new();
            frame.extend_from_slice(&(out.len() as u32).to_be_bytes());
            frame.extend_from_slice(&out);
            sock.write_all(&frame).unwrap();
        });
        let v = identity_echo(&addr, "vm-x", "id1", Duration::from_secs(2)).unwrap();
        assert_eq!(v["vm_id"], "vm-x");
        th.join().unwrap();
    }
}
