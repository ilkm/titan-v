//! UDP LAN discovery broadcast thread (optional multi-homed IPv4 bind).

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use if_addrs::IfAddr;
use titan_common::{CenterPollBeacon, DiscoveryBeacon};

/// Snapshot of discovery settings used to decide whether to respawn the UDP thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoverySpawnSig {
    pub interval_secs: u32,
    pub port: u16,
    pub host_control: String,
    pub bind_ipv4s: Vec<String>,
}

impl DiscoverySpawnSig {
    pub fn new(
        interval_secs: u32,
        port: u16,
        host_control: String,
        mut bind_ipv4s: Vec<String>,
    ) -> Self {
        bind_ipv4s.sort();
        bind_ipv4s.dedup();
        Self {
            interval_secs,
            port,
            host_control,
            bind_ipv4s,
        }
    }
}

/// Snapshot for the center→LAN **host registration poll** UDP thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCollectSpawnSig {
    pub interval_secs: u32,
    pub poll_port: u16,
    pub register_port: u16,
    pub bind_ipv4s: Vec<String>,
}

impl HostCollectSpawnSig {
    pub fn new(
        interval_secs: u32,
        poll_port: u16,
        register_port: u16,
        mut bind_ipv4s: Vec<String>,
    ) -> Self {
        bind_ipv4s.sort();
        bind_ipv4s.dedup();
        Self {
            interval_secs,
            poll_port,
            register_port,
            bind_ipv4s,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanIpv4Row {
    pub ip: Ipv4Addr,
    pub iface: String,
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn is_virtual_iface_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    contains_any(
        &n,
        &[
            "virtual",
            "vmware",
            "vbox",
            "hyper-v",
            "hyperv",
            "vethernet",
            "docker",
            "wsl",
            "npcap",
            "loopback",
            "tunnel",
            "bridge",
            "br-",
            "tap",
            "tun",
            "utun",
            "tailscale",
            "zerotier",
            "wireguard",
            "hamachi",
            "vpn",
        ],
    )
}

/// Non-loopback IPv4 rows for UI (loopback excluded for typical LAN discovery).
pub fn list_lan_ipv4_rows() -> Vec<LanIpv4Row> {
    let mut out = Vec::new();
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return out;
    };
    for i in ifaces {
        if i.is_loopback() {
            continue;
        }
        if is_virtual_iface_name(&i.name) {
            continue;
        }
        let IfAddr::V4(v4) = i.addr else {
            continue;
        };
        if v4.ip.is_unspecified() {
            continue;
        }
        out.push(LanIpv4Row {
            ip: v4.ip,
            iface: i.name,
        });
    }
    out.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.iface.cmp(&b.iface)));
    out.dedup_by(|a, b| a.ip == b.ip && a.iface == b.iface);
    out
}

/// Default IPv4 for manual host entry: same /24-style prefix as the first non-loopback
/// interface, last octet `1` (e.g. machine `192.168.1.100` → `192.168.1.1`).
#[must_use]
pub fn default_manual_host_ipv4_string() -> String {
    let rows = list_lan_ipv4_rows();
    let Some(row) = rows.first() else {
        return "192.168.1.1".to_string();
    };
    let o = row.ip.octets();
    Ipv4Addr::new(o[0], o[1], o[2], 1).to_string()
}

fn ipv4_broadcast_from_mask(addr: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let a = u32::from_be_bytes(addr.octets());
    let m = u32::from_be_bytes(netmask.octets());
    Ipv4Addr::from(((a & m) | !m).to_be_bytes())
}

fn resolve_broadcast_dest(bind: Ipv4Addr, udp_port: u16) -> SocketAddr {
    let dest_ip = if let Ok(ifaces) = if_addrs::get_if_addrs() {
        let mut found = None;
        for i in ifaces {
            let IfAddr::V4(v4) = i.addr else {
                continue;
            };
            if v4.ip == bind {
                found = Some(v4);
                break;
            }
        }
        if let Some(v4) = found {
            v4.broadcast
                .unwrap_or_else(|| ipv4_broadcast_from_mask(v4.ip, v4.netmask))
        } else {
            tracing::warn!(%bind, "discovery: bind IP not found in current interface list; using global broadcast");
            Ipv4Addr::BROADCAST
        }
    } else {
        Ipv4Addr::BROADCAST
    };
    SocketAddr::from((dest_ip, udp_port))
}

use std::net::UdpSocket;

fn udp_push_wildcard_broadcast(
    out: &mut Vec<(UdpSocket, SocketAddr)>,
    dest_port: u16,
    ctx: &'static str,
) -> bool {
    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "{ctx}: UDP bind 0.0.0.0:0 failed");
            return false;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, "{ctx}: set_broadcast failed");
    }
    let Ok(dest) = format!("255.255.255.255:{dest_port}").parse::<SocketAddr>() else {
        return false;
    };
    out.push((sock, dest));
    true
}

fn udp_try_push_iface_bind(
    out: &mut Vec<(UdpSocket, SocketAddr)>,
    bind_s: &str,
    dest_port: u16,
    ctx: &'static str,
) {
    let bind_ip: Ipv4Addr = match bind_s.parse() {
        Ok(ip) => ip,
        Err(_) => {
            tracing::warn!(%bind_s, "{ctx}: skip invalid IPv4 in bind list");
            return;
        }
    };
    let sock = match UdpSocket::bind((bind_ip, 0)) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, %bind_ip, "{ctx}: UDP bind on interface IP failed");
            return;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        tracing::warn!(error = %e, %bind_ip, "{ctx}: set_broadcast failed");
    }
    let dest = resolve_broadcast_dest(bind_ip, dest_port);
    out.push((sock, dest));
}

fn open_udp_broadcast_pairs(
    bind_ipv4s: &[String],
    dest_port: u16,
    ctx: &'static str,
) -> Option<Vec<(UdpSocket, SocketAddr)>> {
    let mut sockets = Vec::new();
    if bind_ipv4s.is_empty() {
        return udp_push_wildcard_broadcast(&mut sockets, dest_port, ctx).then_some(sockets);
    }
    for s in bind_ipv4s {
        udp_try_push_iface_bind(&mut sockets, s, dest_port, ctx);
    }
    if sockets.is_empty() {
        tracing::warn!("{ctx}: no usable bind sockets; falling back to 0.0.0.0");
        let _ = udp_push_wildcard_broadcast(&mut sockets, dest_port, ctx);
    }
    if sockets.is_empty() {
        None
    } else {
        Some(sockets)
    }
}

fn udp_send_payload_to_all(sockets: &[(UdpSocket, SocketAddr)], payload: &[u8], ctx: &'static str) {
    for (sock, dest) in sockets {
        if let Err(e) = sock.send_to(payload, *dest) {
            tracing::debug!(error = %e, %dest, "{ctx}: send_to failed");
        }
    }
}

pub fn discovery_udp_loop(
    my_gen: u64,
    spawn_generation: Arc<AtomicU64>,
    interval: Duration,
    udp_port: u16,
    host_quic_addr: String,
    bind_ipv4s: Vec<String>,
) {
    use std::thread;

    let Some(sockets) = open_udp_broadcast_pairs(&bind_ipv4s, udp_port, "discovery") else {
        return;
    };
    let beacon = DiscoveryBeacon::new(host_quic_addr.clone());
    let payload = match serde_json::to_vec(&beacon) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "discovery: JSON encode failed");
            return;
        }
    };

    loop {
        if spawn_generation.load(Ordering::SeqCst) != my_gen {
            break;
        }
        if !host_quic_addr.trim().is_empty() {
            udp_send_payload_to_all(&sockets, &payload, "discovery");
        }
        thread::sleep(interval);
    }
}

fn host_collect_poll_payload(register_port: u16) -> Option<Vec<u8>> {
    let beacon = CenterPollBeacon::new(register_port);
    match serde_json::to_vec(&beacon) {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!(error = %e, "host_collect: JSON encode failed");
            None
        }
    }
}

fn host_collect_log_startup(poll_port: u16, register_port: u16, interval: Duration) {
    tracing::info!(
        poll_port,
        register_port,
        interval_secs = interval.as_secs(),
        "host_collect: LAN poll for host self-registration (UDP)"
    );
}

/// Periodically broadcasts [`CenterPollBeacon`] so `titan-host serve` nodes reply with [`HostAnnounceBeacon`].
pub fn center_host_collect_udp_loop(
    my_gen: u64,
    spawn_generation: Arc<AtomicU64>,
    interval: Duration,
    poll_port: u16,
    register_port: u16,
    bind_ipv4s: Vec<String>,
) {
    use std::thread;

    let Some(sockets) = open_udp_broadcast_pairs(&bind_ipv4s, poll_port, "host_collect") else {
        return;
    };
    let Some(payload) = host_collect_poll_payload(register_port) else {
        return;
    };
    host_collect_log_startup(poll_port, register_port, interval);

    loop {
        if spawn_generation.load(Ordering::SeqCst) != my_gen {
            break;
        }
        udp_send_payload_to_all(&sockets, &payload, "host_collect");
        thread::sleep(interval);
    }
}
