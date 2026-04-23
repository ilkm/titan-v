//! UDP LAN discovery broadcast thread.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use titan_common::DiscoveryBeacon;

pub fn discovery_udp_loop(
    my_gen: u64,
    gen: Arc<AtomicU64>,
    interval: Duration,
    udp_port: u16,
    host_control: String,
) {
    use std::net::UdpSocket;
    use std::thread;

    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "discovery: UDP bind failed");
            return;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, "discovery: set_broadcast failed");
    }
    let beacon = DiscoveryBeacon::new(host_control.clone());
    let payload = match serde_json::to_vec(&beacon) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "discovery: JSON encode failed");
            return;
        }
    };
    let dest: SocketAddr = match format!("255.255.255.255:{udp_port}").parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    loop {
        if gen.load(Ordering::SeqCst) != my_gen {
            break;
        }
        if !host_control.trim().is_empty() {
            if let Err(e) = sock.send_to(&payload, dest) {
                tracing::debug!(error = %e, "discovery: send_to failed");
            }
        }
        thread::sleep(interval);
    }
}
