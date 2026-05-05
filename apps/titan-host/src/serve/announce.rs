//! LAN registration: reply to Titan Center **poll** UDP beacons; optional periodic announce.
//!
//! v3 of [`HostAnnounceBeacon`] carries the host's mTLS SPKI fingerprint (`host_spki_sha256_hex`)
//! so Center can auto-trust it without a TOFU prompt for LAN-discovered devices.

use std::net::SocketAddr;
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

pub fn resolve_public_quic_addr(
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

fn build_announce_payload(
    public: &str,
    label: &str,
    device_id: &str,
    fingerprint: &str,
) -> Option<Vec<u8>> {
    let beacon = HostAnnounceBeacon::new(public, label, device_id, fingerprint);
    to_vec(&beacon).ok()
}

fn host_announce_payload(
    cfg: &HostAnnounceConfig,
    bind_request: SocketAddr,
    local: SocketAddr,
    identity: &Identity,
) -> Option<(String, String, String, Vec<u8>)> {
    let public = resolve_public_quic_addr(bind_request, local, cfg.public_addr_override.as_deref());
    let label = cfg.label_override.clone().unwrap_or_else(|| {
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string())
    });
    let device_id = crate::host_device_id::host_device_id_string();
    let payload = build_announce_payload(&public, &label, &device_id, &identity.spki_sha256_hex)?;
    Some((public, label, device_id, payload))
}

pub fn spawn_host_announce_background(
    cfg: HostAnnounceConfig,
    bind_request: SocketAddr,
    local: SocketAddr,
    identity: &Arc<Identity>,
) {
    if !cfg.enabled {
        return;
    }
    let Some((public, label, device_id, payload)) =
        host_announce_payload(&cfg, bind_request, local, identity)
    else {
        tracing::warn!("host announce: JSON encode failed");
        return;
    };

    tracing::info!(
        addr = %public,
        label = %label,
        device_id = %device_id,
        fingerprint = %identity.spki_sha256_hex,
        poll_listen_port = cfg.center_poll_listen_port,
        register_port = cfg.center_register_udp_port,
        periodic = ?cfg.periodic_interval,
        "host announce: LAN registration (center poll + optional periodic)"
    );
    sidecars::start_announce_sidecars(&cfg, payload);
}
