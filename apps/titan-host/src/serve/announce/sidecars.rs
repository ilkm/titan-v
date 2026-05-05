use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

use serde_json::from_slice;
use titan_common::CenterPollBeacon;

use super::HostAnnounceConfig;

const INITIAL_BURST_INTERVAL: Duration = Duration::from_millis(50);
const INITIAL_BURST_COUNT: u32 = 10;

pub(super) fn start_announce_sidecars(cfg: &HostAnnounceConfig, payload: Vec<u8>) {
    if cfg.center_poll_listen_port > 0 {
        spawn_center_poll_responder(cfg.center_poll_listen_port, payload.clone());
    }
    spawn_initial_burst_announce(cfg.center_register_udp_port, payload.clone());
    if let Some(iv) = cfg.periodic_interval
        && !iv.is_zero()
    {
        spawn_periodic_announce(iv, cfg.center_register_udp_port, payload);
    }
}

fn spawn_periodic_announce(interval: Duration, center_register_udp_port: u16, payload: Vec<u8>) {
    let Some(dest) = broadcast_dest(center_register_udp_port) else {
        return;
    };
    thread::spawn(move || {
        let Some(sock) = build_broadcast_socket("periodic") else {
            return;
        };
        loop {
            if let Err(e) = sock.send_to(&payload, dest) {
                tracing::debug!(error = %e, "host announce: periodic send_to failed");
            }
            thread::sleep(interval);
        }
    });
}

fn spawn_initial_burst_announce(center_register_udp_port: u16, payload: Vec<u8>) {
    let Some(dest) = broadcast_dest(center_register_udp_port) else {
        return;
    };
    thread::spawn(move || {
        let Some(sock) = build_broadcast_socket("burst") else {
            return;
        };
        for _ in 0..INITIAL_BURST_COUNT {
            if let Err(e) = sock.send_to(&payload, dest) {
                tracing::debug!(error = %e, "host announce: burst send_to failed");
            }
            thread::sleep(INITIAL_BURST_INTERVAL);
        }
    });
}

fn spawn_center_poll_responder(listen_port: u16, payload: Vec<u8>) {
    thread::spawn(move || {
        let sock = match UdpSocket::bind(("0.0.0.0", listen_port)) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    port = listen_port,
                    "host announce: poll listen bind failed (port in use?)"
                );
                return;
            }
        };
        if let Err(e) = sock.set_broadcast(true) {
            tracing::warn!(error = %e, "host announce: poll set_broadcast failed");
        }
        tracing::info!(
            port = listen_port,
            "host announce: listening for Titan Center LAN registration polls (UDP)"
        );
        let mut buf = vec![0u8; 4096];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, peer)) => center_poll_try_reply(&sock, &buf, n, peer, &payload),
                Err(e) => tracing::debug!(error = %e, "host announce: poll recv"),
            }
        }
    });
}

fn center_poll_try_reply(sock: &UdpSocket, buf: &[u8], n: usize, peer: SocketAddr, payload: &[u8]) {
    let poll: CenterPollBeacon = match from_slice(&buf[..n]) {
        Ok(b) => b,
        Err(_) => return,
    };
    if poll.validate().is_err() {
        return;
    }
    let dest = SocketAddr::new(peer.ip(), poll.register_udp_port);
    if let Err(e) = sock.send_to(payload, dest) {
        tracing::debug!(error = %e, %dest, "host announce: poll reply send_to failed");
    }
}

fn broadcast_dest(port: u16) -> Option<SocketAddr> {
    format!("255.255.255.255:{port}").parse().ok()
}

fn build_broadcast_socket(label: &'static str) -> Option<UdpSocket> {
    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, label, "host announce: UDP bind failed");
            return None;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, label, "host announce: set_broadcast failed");
    }
    Some(sock)
}
