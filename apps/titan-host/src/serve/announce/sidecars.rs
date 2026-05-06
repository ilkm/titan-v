use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use if_addrs::IfAddr;
use serde_json::from_slice;
use titan_common::CenterPollBeacon;

use super::{AnnounceEndpoint, HostAnnounceConfig};

const INITIAL_BURST_INTERVAL: Duration = Duration::from_millis(50);
const INITIAL_BURST_COUNT: u32 = 10;
const POLL_RECV_TIMEOUT: Duration = Duration::from_millis(300);
static ANNOUNCE_SIDECAR_GEN: AtomicU64 = AtomicU64::new(0);

pub(super) fn start_announce_sidecars(cfg: &HostAnnounceConfig, endpoints: Vec<AnnounceEndpoint>) {
    let my_gen = ANNOUNCE_SIDECAR_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    if cfg.center_poll_listen_port > 0 {
        spawn_center_poll_responder(my_gen, cfg.center_poll_listen_port, endpoints.clone());
    }
    spawn_initial_burst_announce(my_gen, cfg.center_register_udp_port, endpoints.clone());
    if let Some(iv) = cfg.periodic_interval
        && !iv.is_zero()
    {
        spawn_periodic_announce(my_gen, iv, cfg.center_register_udp_port, endpoints);
    }
}

fn spawn_periodic_announce(
    my_gen: u64,
    interval: Duration,
    center_register_udp_port: u16,
    endpoints: Vec<AnnounceEndpoint>,
) {
    let pairs = announce_pairs_for_broadcast(center_register_udp_port, &endpoints);
    if pairs.is_empty() {
        return;
    }
    thread::spawn(move || {
        loop {
            if ANNOUNCE_SIDECAR_GEN.load(Ordering::SeqCst) != my_gen {
                break;
            }
            send_pairs(&pairs, "periodic");
            thread::sleep(interval);
        }
    });
}

fn spawn_initial_burst_announce(
    my_gen: u64,
    center_register_udp_port: u16,
    endpoints: Vec<AnnounceEndpoint>,
) {
    let pairs = announce_pairs_for_broadcast(center_register_udp_port, &endpoints);
    if pairs.is_empty() {
        return;
    }
    thread::spawn(move || {
        for _ in 0..INITIAL_BURST_COUNT {
            if ANNOUNCE_SIDECAR_GEN.load(Ordering::SeqCst) != my_gen {
                break;
            }
            send_pairs(&pairs, "burst");
            thread::sleep(INITIAL_BURST_INTERVAL);
        }
    });
}

fn spawn_center_poll_responder(my_gen: u64, listen_port: u16, endpoints: Vec<AnnounceEndpoint>) {
    let pairs = announce_pairs_for_reply(&endpoints);
    if pairs.is_empty() {
        return;
    }
    thread::spawn(move || {
        let Some(sock) = build_poll_listen_socket(listen_port) else {
            return;
        };
        let mut buf = vec![0u8; 4096];
        loop {
            if ANNOUNCE_SIDECAR_GEN.load(Ordering::SeqCst) != my_gen {
                break;
            }
            match sock.recv_from(&mut buf) {
                Ok((n, peer)) => center_poll_try_reply(&buf, n, peer, &pairs),
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => tracing::debug!(error = %e, "host announce: poll recv"),
            }
        }
    });
}

fn build_poll_listen_socket(listen_port: u16) -> Option<UdpSocket> {
    let sock = match UdpSocket::bind(("0.0.0.0", listen_port)) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                error = %e,
                port = listen_port,
                "host announce: poll listen bind failed (port in use?)"
            );
            return None;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, "host announce: poll set_broadcast failed");
    }
    if let Err(e) = sock.set_read_timeout(Some(POLL_RECV_TIMEOUT)) {
        tracing::warn!(error = %e, "host announce: poll set_read_timeout failed");
    }
    tracing::info!(
        port = listen_port,
        "host announce: listening for Titan Center LAN registration polls (UDP)"
    );
    Some(sock)
}

fn center_poll_try_reply(buf: &[u8], n: usize, peer: SocketAddr, pairs: &[AnnouncePair]) {
    let poll: CenterPollBeacon = match from_slice(&buf[..n]) {
        Ok(b) => b,
        Err(_) => return,
    };
    if poll.validate().is_err() {
        return;
    }
    let dest = SocketAddr::new(peer.ip(), poll.register_udp_port);
    for pair in choose_reply_pairs(peer, pairs) {
        if let Err(e) = pair.sock.send_to(&pair.payload, dest) {
            tracing::debug!(error = %e, %dest, "host announce: poll reply send_to failed");
        }
    }
}

struct AnnouncePair {
    bind_ip: Ipv4Addr,
    netmask: Ipv4Addr,
    sock: UdpSocket,
    payload: Vec<u8>,
}

fn announce_pairs_for_broadcast(port: u16, endpoints: &[AnnounceEndpoint]) -> Vec<AnnouncePair> {
    endpoints
        .iter()
        .filter_map(|e| build_announce_pair(e, Some(port), "broadcast"))
        .collect()
}

fn announce_pairs_for_reply(endpoints: &[AnnounceEndpoint]) -> Vec<AnnouncePair> {
    endpoints
        .iter()
        .filter_map(|e| build_announce_pair(e, None, "reply"))
        .collect()
}

fn build_announce_pair(
    endpoint: &AnnounceEndpoint,
    broadcast_port: Option<u16>,
    label: &'static str,
) -> Option<AnnouncePair> {
    let (bind_ip, netmask) = endpoint_bind_and_mask(endpoint.bind_ip);
    let sock = build_bound_socket(bind_ip, label)?;
    let payload = endpoint.payload.clone();
    if let Some(port) = broadcast_port {
        let dest = broadcast_dest(bind_ip, netmask, port);
        if let Err(e) = sock.connect(dest) {
            tracing::warn!(error = %e, %dest, "host announce: connect broadcast destination failed");
            return None;
        }
    }
    Some(AnnouncePair {
        bind_ip,
        netmask,
        sock,
        payload,
    })
}

fn endpoint_bind_and_mask(bind_ip: Ipv4Addr) -> (Ipv4Addr, Ipv4Addr) {
    if bind_ip.is_unspecified() {
        return (Ipv4Addr::UNSPECIFIED, Ipv4Addr::new(0, 0, 0, 0));
    }
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return (bind_ip, Ipv4Addr::new(255, 255, 255, 255));
    };
    for iface in ifaces {
        let IfAddr::V4(v4) = iface.addr else {
            continue;
        };
        if v4.ip == bind_ip {
            return (bind_ip, v4.netmask);
        }
    }
    (bind_ip, Ipv4Addr::new(255, 255, 255, 255))
}

fn build_bound_socket(bind_ip: Ipv4Addr, label: &'static str) -> Option<UdpSocket> {
    let bind = SocketAddr::new(bind_ip.into(), 0);
    let sock = match UdpSocket::bind(bind) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, %bind, label, "host announce: UDP bind failed");
            return None;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, label, "host announce: set_broadcast failed");
    }
    Some(sock)
}

fn send_pairs(pairs: &[AnnouncePair], label: &'static str) {
    for pair in pairs {
        if let Err(e) = pair.sock.send(&pair.payload) {
            tracing::debug!(error = %e, label, "host announce: send failed");
        }
    }
}

fn choose_reply_pairs(peer: SocketAddr, pairs: &[AnnouncePair]) -> Vec<&AnnouncePair> {
    let SocketAddr::V4(peer_v4) = peer else {
        return pairs.iter().collect();
    };
    let peer_ip = *peer_v4.ip();
    let same_subnet: Vec<&AnnouncePair> = pairs
        .iter()
        .filter(|pair| ipv4_in_subnet(peer_ip, pair.bind_ip, pair.netmask))
        .collect();
    if same_subnet.is_empty() {
        pairs.iter().collect()
    } else {
        same_subnet
    }
}

fn broadcast_dest(bind_ip: Ipv4Addr, netmask: Ipv4Addr, port: u16) -> SocketAddr {
    if bind_ip.is_unspecified() {
        return SocketAddr::new(Ipv4Addr::BROADCAST.into(), port);
    }
    let ip = ipv4_broadcast_from_mask(bind_ip, netmask);
    SocketAddr::new(ip.into(), port)
}

fn ipv4_broadcast_from_mask(addr: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let a = u32::from_be_bytes(addr.octets());
    let m = u32::from_be_bytes(netmask.octets());
    Ipv4Addr::from(((a & m) | !m).to_be_bytes())
}

fn ipv4_in_subnet(target: Ipv4Addr, iface_ip: Ipv4Addr, netmask: Ipv4Addr) -> bool {
    let t = u32::from_be_bytes(target.octets());
    let i = u32::from_be_bytes(iface_ip.octets());
    let m = u32::from_be_bytes(netmask.octets());
    (t & m) == (i & m)
}
