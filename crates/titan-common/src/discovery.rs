//! LAN discovery beacons (UDP JSON).
//!
//! Two related broadcast flows live here:
//!
//! 1. **Center↔Host registration** — `CenterPollBeacon` (Center→LAN) and `HostAnnounceBeacon`
//!    (Host→Center). The host announce includes the host's self-signed mTLS SPKI fingerprint
//!    so Center can auto-trust it on first sight.
//! 2. **In-guest helper** — `DiscoveryBeacon` (Center→VM-internal automation) advertises the
//!    host's QUIC control endpoint so guest scripts can find it without hard-coding addresses.

use serde::{Deserialize, Serialize};

/// JSON `kind` for [`DiscoveryBeacon`]; in-guest automation may listen for it.
pub const DISCOVERY_BEACON_KIND: &str = "titan.v1.discovery";

/// Schema for [`DiscoveryBeacon`]; bump when fields change incompatibly.
pub const DISCOVERY_SCHEMA_VERSION: u32 = 1;

/// Default UDP destination port for [`DiscoveryBeacon`] (Center→VM-internal).
pub const DEFAULT_DISCOVERY_UDP_PORT: u16 = 7789;

/// Payload sent as UTF-8 JSON over UDP from Center to in-guest automation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryBeacon {
    pub kind: String,
    pub schema: u32,
    /// QUIC control endpoint for the host this beacon advertises (e.g. `192.168.1.10:7788`).
    pub host_quic_addr: String,
}

impl DiscoveryBeacon {
    #[must_use]
    pub fn new(host_quic_addr: impl Into<String>) -> Self {
        Self {
            kind: DISCOVERY_BEACON_KIND.to_string(),
            schema: DISCOVERY_SCHEMA_VERSION,
            host_quic_addr: host_quic_addr.into(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != DISCOVERY_BEACON_KIND {
            return Err("discovery beacon kind mismatch");
        }
        if self.schema != DISCOVERY_SCHEMA_VERSION {
            return Err("discovery beacon schema mismatch");
        }
        if self.host_quic_addr.trim().is_empty() {
            return Err("empty host_quic_addr");
        }
        Ok(())
    }
}

/// JSON `kind` for [`HostAnnounceBeacon`]; center listens on UDP and merges into device list.
pub const HOST_ANNOUNCE_BEACON_KIND: &str = "titan.v3.host_announce";

/// Schema for [`HostAnnounceBeacon`]. v3 is the **only** accepted schema; QUIC + mTLS by default.
pub const HOST_ANNOUNCE_SCHEMA_VERSION: u32 = 3;

/// Default UDP port **Titan Center** binds for LAN host registration (host sends to this port).
pub const DEFAULT_CENTER_REGISTER_UDP_PORT: u16 = 7791;

/// Payload sent by `titan-host serve` so Titan Center can auto-add the device on the LAN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostAnnounceBeacon {
    pub kind: String,
    pub schema: u32,
    /// QUIC endpoint clients dial (UDP), e.g. `192.168.1.10:7788`.
    pub host_quic_addr: String,
    /// Display label in the center device list (may be empty; center synthesizes one).
    pub label: String,
    /// OS-stable machine id from `titan-host` (`machine-uid`); never empty.
    pub device_id: String,
    /// SHA-256(SPKI(host cert)) hex (lowercase, 64 chars). Center trust store keys on this.
    pub host_spki_sha256_hex: String,
}

impl HostAnnounceBeacon {
    #[must_use]
    pub fn new(
        host_quic_addr: impl Into<String>,
        label: impl Into<String>,
        device_id: impl Into<String>,
        host_spki_sha256_hex: impl Into<String>,
    ) -> Self {
        Self {
            kind: HOST_ANNOUNCE_BEACON_KIND.to_string(),
            schema: HOST_ANNOUNCE_SCHEMA_VERSION,
            host_quic_addr: host_quic_addr.into(),
            label: label.into(),
            device_id: device_id.into(),
            host_spki_sha256_hex: host_spki_sha256_hex.into(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != HOST_ANNOUNCE_BEACON_KIND {
            return Err("host announce beacon kind mismatch");
        }
        if self.schema != HOST_ANNOUNCE_SCHEMA_VERSION {
            return Err("host announce beacon schema mismatch");
        }
        if self.host_quic_addr.trim().is_empty() {
            return Err("empty host_quic_addr");
        }
        if self.device_id.trim().is_empty() {
            return Err("empty device_id");
        }
        if !is_lowercase_hex_64(&self.host_spki_sha256_hex) {
            return Err("host_spki_sha256_hex must be 64 lowercase hex chars");
        }
        Ok(())
    }
}

fn is_lowercase_hex_64(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

/// JSON `kind` for [`CenterPollBeacon`]; hosts listen on [`DEFAULT_CENTER_POLL_UDP_PORT`] UDP.
pub const CENTER_POLL_BEACON_KIND: &str = "titan.v3.center_poll";

/// Schema for [`CenterPollBeacon`]; v3 only.
pub const CENTER_POLL_SCHEMA_VERSION: u32 = 3;

/// UDP **destination** port for center→LAN poll packets; `titan-host serve` binds this port to listen.
pub const DEFAULT_CENTER_POLL_UDP_PORT: u16 = 7792;

/// Center periodically broadcasts this so hosts reply with [`HostAnnounceBeacon`] to `register_udp_port`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CenterPollBeacon {
    pub kind: String,
    pub schema: u32,
    /// Send [`HostAnnounceBeacon`] to `(source_ip_of_this_packet, register_udp_port)`.
    pub register_udp_port: u16,
    /// SHA-256(SPKI(center cert)) hex so hosts can auto-trust Center on LAN poll.
    pub center_spki_sha256_hex: String,
}

impl CenterPollBeacon {
    #[must_use]
    pub fn new(register_udp_port: u16, center_spki_sha256_hex: impl Into<String>) -> Self {
        Self {
            kind: CENTER_POLL_BEACON_KIND.to_string(),
            schema: CENTER_POLL_SCHEMA_VERSION,
            register_udp_port,
            center_spki_sha256_hex: center_spki_sha256_hex.into(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != CENTER_POLL_BEACON_KIND {
            return Err("center poll beacon kind mismatch");
        }
        if self.schema != CENTER_POLL_SCHEMA_VERSION {
            return Err("center poll beacon schema mismatch");
        }
        if self.register_udp_port == 0 {
            return Err("register_udp_port must be non-zero");
        }
        if !is_lowercase_hex_64(&self.center_spki_sha256_hex) {
            return Err("center_spki_sha256_hex must be 64 lowercase hex chars");
        }
        Ok(())
    }
}

#[cfg(test)]
mod announce_tests {
    use super::*;

    fn fake_fp() -> String {
        "0".repeat(64)
    }

    #[test]
    fn host_announce_roundtrip() {
        let b = HostAnnounceBeacon::new("192.168.1.2:7788", "rack-a", "machine-id-hex", fake_fp());
        let v = serde_json::to_vec(&b).unwrap();
        let d: HostAnnounceBeacon = serde_json::from_slice(&v).unwrap();
        assert_eq!(d, b);
        d.validate().unwrap();
    }

    #[test]
    fn host_announce_rejects_short_fingerprint() {
        let b = HostAnnounceBeacon::new("10.0.0.2:7788", "h", "mid", "abc");
        assert!(b.validate().is_err());
    }

    #[test]
    fn host_announce_rejects_v2_schema() {
        let json = br#"{"kind":"titan.v3.host_announce","schema":2,"host_quic_addr":"10.0.0.2:7788","label":"h","device_id":"mid","host_spki_sha256_hex":"00000000000000000000000000000000000000000000000000000000000000aa"}"#;
        let d: HostAnnounceBeacon = serde_json::from_slice(json).unwrap();
        assert!(d.validate().is_err());
    }

    #[test]
    fn center_poll_roundtrip() {
        let b = CenterPollBeacon::new(7791, fake_fp());
        let v = serde_json::to_vec(&b).unwrap();
        let d: CenterPollBeacon = serde_json::from_slice(&v).unwrap();
        assert_eq!(d, b);
        d.validate().unwrap();
    }
}
