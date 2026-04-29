//! `socket2` + Tokio: tuned TCP listeners for the control plane.

use std::io;
use std::net::SocketAddr;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::TcpListener;

/// Bind a non-blocking TCP listener with reuseaddr (where supported) for command/telemetry ports.
pub fn tcp_listen_tokio(addr: SocketAddr) -> io::Result<TcpListener> {
    let domain = Domain::for_address(addr);
    let sock = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    sock.set_nonblocking(true)?;
    #[cfg(not(target_os = "solaris"))]
    {
        let _ = sock.set_reuse_address(true);
    }
    sock.bind(&addr.into())?;
    sock.listen(1024)?;
    let std_listener: std::net::TcpListener = sock.into();
    TcpListener::from_std(std_listener)
}
