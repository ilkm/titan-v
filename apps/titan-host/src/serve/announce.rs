//! LAN registration: reply to Titan Center **poll** UDP beacons; optional periodic announce.
//!
//! v3 of [`HostAnnounceBeacon`] carries the host's mTLS SPKI fingerprint (`host_spki_sha256_hex`)
//! so Center can auto-trust it without a TOFU prompt for LAN-discovered devices.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

mod sidecars;

use serde_json::to_vec;
use titan_common::{
    DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT, HostAnnounceBeacon,
};
use titan_quic::Identity;

/// CLI / launch-time options; [`run_serve`](super::run::run_serve) fills public control addr / label before spawning.
#[derive(Clone, Debug)]
pub struct HostAnnounceConfig {
    pub enabled: bool,
    /// When `Some`, also broadcast [`HostAnnounceBeacon`] on this interval (in addition to poll replies).
    pub periodic_interval: Option<Duration>,
    pub center_register_udp_port: u16,
    /// Listen here for [`CenterPollBeacon`] from Titan Center (UDP).
    pub center_poll_listen_port: u16,
    pub bind_ipv4: Option<Ipv4Addr>,
    pub public_addr_override: Option<String>,
    pub label_override: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LanIpv4Row {
    pub ip: Ipv4Addr,
    pub iface: String,
}

#[derive(Clone, Debug)]
pub(super) struct AnnounceEndpoint {
    pub bind_ip: Ipv4Addr,
    pub payload: Vec<u8>,
}

impl Default for HostAnnounceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            periodic_interval: None,
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            bind_ipv4: None,
            public_addr_override: None,
            label_override: None,
        }
    }
}

fn resolve_public_override(override_addr: Option<&str>) -> Option<String> {
    let s = override_addr?.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn build_announce_payload(
    public: &str,
    label: &str,
    device_id: &str,
    fingerprint: &str,
) -> Option<Vec<u8>> {
    let beacon = HostAnnounceBeacon::new(public, label, device_id, fingerprint);
    to_vec(&beacon).ok()
}

fn host_announce_payloads(
    cfg: &HostAnnounceConfig,
    local: SocketAddr,
    identity: &Identity,
) -> Option<(String, String, String, Vec<AnnounceEndpoint>)> {
    let label = cfg.label_override.clone().unwrap_or_else(|| {
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string())
    });
    let device_id = crate::host_device_id::host_device_id_string();
    let bind_ipv4s = resolve_bind_ipv4s(cfg.bind_ipv4);
    if bind_ipv4s.is_empty() {
        return None;
    }
    build_bind_scoped_payloads(
        local,
        &label,
        &device_id,
        &identity.spki_sha256_hex,
        cfg.public_addr_override.as_deref(),
        &bind_ipv4s,
    )
    .map(|(primary, endpoints)| (primary, label, device_id, endpoints))
}

fn build_bind_scoped_payloads(
    local: SocketAddr,
    label: &str,
    device_id: &str,
    fingerprint: &str,
    override_addr: Option<&str>,
    bind_ipv4s: &[Ipv4Addr],
) -> Option<(String, Vec<AnnounceEndpoint>)> {
    let override_public = resolve_public_override(override_addr);
    let mut endpoints = Vec::new();
    for ip in bind_ipv4s {
        let public = override_public
            .clone()
            .unwrap_or_else(|| format!("{ip}:{}", local.port()));
        let payload = build_announce_payload(&public, label, device_id, fingerprint)?;
        endpoints.push(AnnounceEndpoint {
            bind_ip: *ip,
            payload,
        });
    }
    let primary = override_public.unwrap_or_else(|| format!("{}:{}", bind_ipv4s[0], local.port()));
    Some((primary, endpoints))
}

fn resolve_bind_ipv4s(selected: Option<Ipv4Addr>) -> Vec<Ipv4Addr> {
    let all = resolve_physical_ipv4s();
    if let Some(ip) = selected {
        if all.contains(&ip) {
            return vec![ip];
        }
        tracing::warn!(%ip, "host announce: selected LAN bind IPv4 not available");
        return Vec::new();
    }
    all.first().copied().map(|ip| vec![ip]).unwrap_or_default()
}

pub fn list_physical_lan_ipv4_rows() -> Vec<LanIpv4Row> {
    let mut rows = Vec::new();
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return rows;
    };
    for iface in ifaces {
        if iface.is_loopback() || is_virtual_iface_name(&iface.name) {
            continue;
        }
        let if_addrs::IfAddr::V4(v4) = iface.addr else {
            continue;
        };
        if v4.ip.is_unspecified() {
            continue;
        }
        rows.push(LanIpv4Row {
            ip: v4.ip,
            iface: iface.name,
        });
    }
    rows.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.iface.cmp(&b.iface)));
    rows.dedup_by(|a, b| a.ip == b.ip && a.iface == b.iface);
    rows
}

pub(crate) fn resolve_physical_ipv4s() -> Vec<Ipv4Addr> {
    let mut out: Vec<Ipv4Addr> = list_physical_lan_ipv4_rows()
        .into_iter()
        .map(|row| row.ip)
        .collect();
    out.sort();
    out.dedup();
    out
}

fn is_virtual_iface_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [
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
    ]
    .iter()
    .any(|needle| n.contains(needle))
}

pub fn spawn_host_announce_background(
    cfg: HostAnnounceConfig,
    _bind_request: SocketAddr,
    local: SocketAddr,
    identity: &Arc<Identity>,
) {
    if !cfg.enabled {
        return;
    }
    let Some((public, label, device_id, endpoints)) = host_announce_payloads(&cfg, local, identity)
    else {
        tracing::warn!("host announce: no usable physical IPv4 address for LAN registration");
        return;
    };

    tracing::info!(
        addr = %public,
        label = %label,
        device_id = %device_id,
        fingerprint = %identity.spki_sha256_hex,
        poll_listen_port = cfg.center_poll_listen_port,
        register_port = cfg.center_register_udp_port,
        endpoint_count = endpoints.len(),
        periodic = ?cfg.periodic_interval,
        "host announce: LAN registration (center poll + optional periodic)"
    );
    sidecars::start_announce_sidecars(&cfg, endpoints);
}
