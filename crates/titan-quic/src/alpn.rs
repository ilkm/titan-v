//! ALPN identifiers for Titan-v QUIC.
//!
//! Two protocols share one QUIC connection:
//! * [`ALPN_CONTROL_V1`] — Center→Host bi-streams: one RPC per stream (rkyv-encoded
//!   `ControlRequestFrame` request → `ControlHostFrame::Response`).
//! * [`ALPN_TELEMETRY_V1`] — Host→Center single uni-stream: append-only `ControlPush` frames.
//!
//! Bumping the wire-level [`titan_common::PROTOCOL_VERSION`] alone does **not** change ALPN;
//! ALPN moves only on a hard transport break (e.g. dropping rkyv).

pub const ALPN_CONTROL_V1: &[u8] = b"titan-control-v1";
pub const ALPN_TELEMETRY_V1: &[u8] = b"titan-telemetry-v1";

#[must_use]
pub fn alpn_protocols_server() -> Vec<Vec<u8>> {
    vec![ALPN_CONTROL_V1.to_vec(), ALPN_TELEMETRY_V1.to_vec()]
}

#[must_use]
pub fn alpn_protocols_client_control() -> Vec<Vec<u8>> {
    vec![ALPN_CONTROL_V1.to_vec()]
}
