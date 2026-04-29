//! UDP broadcast to Titan Center (same port as LAN host registration).

use std::net::{SocketAddr, UdpSocket};

use titan_common::VmWindowRegisterBeacon;

pub(crate) fn spawn_vm_window_register_beacon(beacon: VmWindowRegisterBeacon, register_port: u16) {
    let Ok(payload) = serde_json::to_vec(&beacon) else {
        return;
    };
    std::thread::spawn(move || {
        let dest: SocketAddr = match format!("255.255.255.255:{register_port}").parse() {
            Ok(a) => a,
            Err(_) => return,
        };
        let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else {
            return;
        };
        let _ = sock.set_broadcast(true);
        if let Err(e) = sock.send_to(&payload, dest) {
            tracing::debug!(error = %e, "vm_window UDP notify send_to failed");
        }
    });
}
