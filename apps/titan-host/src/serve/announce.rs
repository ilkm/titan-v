//! LAN registration: reply to Titan Center **poll** UDP beacons; optional periodic announce.

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

use serde_json::{from_slice, to_vec};
use titan_common::{
    CenterPollBeacon, DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT,
    HostAnnounceBeacon,
};

/// CLI / launch-time options; [`run_serve`](super::run::run_serve) fills public control addr / label before spawning.
#[derive(Clone, Debug)]
pub struct HostAnnounceConfig {
    pub enabled: bool,
    /// When `Some`, also broadcast [`HostAnnounceBeacon`] on this interval (in addition to poll replies).
    pub periodic_interval: Option<Duration>,
    pub center_register_udp_port: u16,
    /// Listen here for [`CenterPollBeacon`] from Titan Center (UDP).
    pub center_poll_listen_port: u16,
    pub public_addr_override: Option<String>,
    pub label_override: Option<String>,
}

impl Default for HostAnnounceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            periodic_interval: None,
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            public_addr_override: None,
            label_override: None,
        }
    }
}

/// Picks `IP:port` for the beacon when `--listen 0.0.0.0:N` (first non-loopback IPv4 + `local.port()`).
pub fn resolve_public_control_addr(
    _bind_request: SocketAddr,
    local: SocketAddr,
    override_addr: Option<&str>,
) -> String {
    if let Some(o) = override_addr {
        let s = o.trim();
        if !s.is_empty() {
            return s.to_string();
        }
    }
    let port = local.port();
    if let Ok(ifaces) = if_addrs::get_if_addrs() {
        for i in ifaces {
            if i.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = i.addr
                && !v4.ip.is_unspecified()
            {
                return format!("{}:{}", v4.ip, port);
            }
        }
    }
    format!("127.0.0.1:{port}")
}

fn build_announce_payload(public: &str, label: &str, device_id: &str) -> Option<Vec<u8>> {
    let beacon = HostAnnounceBeacon::new(public, label, device_id);
    to_vec(&beacon).ok()
}

fn spawn_periodic_announce(interval: Duration, center_register_udp_port: u16, payload: Vec<u8>) {
    let dest: SocketAddr = match format!("255.255.255.255:{center_register_udp_port}").parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    thread::spawn(move || {
        let sock = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "host announce: periodic UDP bind failed");
                return;
            }
        };
        if let Err(e) = sock.set_broadcast(true) {
            tracing::warn!(error = %e, "host announce: periodic set_broadcast failed");
        }
        loop {
            if let Err(e) = sock.send_to(&payload, dest) {
                tracing::debug!(error = %e, "host announce: periodic send_to failed");
            }
            thread::sleep(interval);
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

fn host_announce_payload(
    cfg: &HostAnnounceConfig,
    bind_request: SocketAddr,
    local: SocketAddr,
) -> Option<(String, String, String, Vec<u8>)> {
    let public =
        resolve_public_control_addr(bind_request, local, cfg.public_addr_override.as_deref());
    let label = cfg.label_override.clone().unwrap_or_else(|| {
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string())
    });
    let device_id = crate::host_device_id::host_device_id_string();
    let payload = build_announce_payload(&public, &label, &device_id)?;
    Some((public, label, device_id, payload))
}

fn start_announce_sidecars(cfg: &HostAnnounceConfig, payload: Vec<u8>) {
    if cfg.center_poll_listen_port > 0 {
        spawn_center_poll_responder(cfg.center_poll_listen_port, payload.clone());
    }
    if let Some(iv) = cfg.periodic_interval
        && !iv.is_zero()
    {
        spawn_periodic_announce(iv, cfg.center_register_udp_port, payload);
    }
}

pub fn spawn_host_announce_background(
    cfg: HostAnnounceConfig,
    bind_request: SocketAddr,
    local: SocketAddr,
) {
    if !cfg.enabled {
        return;
    }
    let Some((public, label, device_id, payload)) =
        host_announce_payload(&cfg, bind_request, local)
    else {
        tracing::warn!("host announce: JSON encode failed");
        return;
    };

    tracing::info!(
        addr = %public,
        label = %label,
        device_id = %device_id,
        poll_listen_port = cfg.center_poll_listen_port,
        register_port = cfg.center_register_udp_port,
        periodic = ?cfg.periodic_interval,
        "host announce: LAN registration (center poll + optional periodic)"
    );
    start_announce_sidecars(&cfg, payload);
}
