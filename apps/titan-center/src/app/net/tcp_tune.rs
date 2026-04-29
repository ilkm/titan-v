//! Post-connect tuning for Tokio `TcpStream` (control plane client).
//!
//! Tokio exposes `set_nodelay` directly; buffer sizing would require `into_std`/`from_std`
//! or a newer Tokio `socket_ref` API — keep this helper for one place to extend later.

use std::io;

use tokio::net::TcpStream;

/// Apply TCP_NODELAY on an established control-plane stream.
pub fn tune_connected_stream(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)
}
