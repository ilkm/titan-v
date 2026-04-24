//! LAN discovery beacon (UDP JSON): center broadcasts where the **host control plane** listens.

use serde::{Deserialize, Serialize};

/// JSON `kind` for [`DiscoveryBeacon`]; guests must reject unknown kinds.
pub const DISCOVERY_BEACON_KIND: &str = "titan.v1.discovery";

/// Schema carried in [`DiscoveryBeacon::schema`]; bump when fields change incompatibly.
pub const DISCOVERY_SCHEMA_VERSION: u32 = 1;

/// Default UDP destination port for beacons (optional in-guest automation may listen here).
pub const DEFAULT_DISCOVERY_UDP_PORT: u16 = 7789;

/// Payload sent as UTF-8 JSON over UDP (e.g. to `255.255.255.255:7789`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryBeacon {
    pub kind: String,
    pub schema: u32,
    /// `titan-host serve` control-plane TCP address, e.g. `192.168.1.10:7788`.
    pub host_control_addr: String,
}

impl DiscoveryBeacon {
    #[must_use]
    pub fn new(host_control_addr: impl Into<String>) -> Self {
        Self {
            kind: DISCOVERY_BEACON_KIND.to_string(),
            schema: DISCOVERY_SCHEMA_VERSION,
            host_control_addr: host_control_addr.into(),
        }
    }

    /// Validates `kind` / `schema` before trusting `host_control_addr`.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != DISCOVERY_BEACON_KIND {
            return Err("discovery beacon kind mismatch");
        }
        if self.schema != DISCOVERY_SCHEMA_VERSION {
            return Err("discovery beacon schema mismatch");
        }
        if self.host_control_addr.trim().is_empty() {
            return Err("empty host_control_addr");
        }
        Ok(())
    }
}

/// JSON `kind` for [`HostAnnounceBeacon`]; center listens on UDP and merges into device list.
pub const HOST_ANNOUNCE_BEACON_KIND: &str = "titan.v1.host_announce";

/// Schema for [`HostAnnounceBeacon`].
pub const HOST_ANNOUNCE_SCHEMA_VERSION: u32 = 2;

/// Default UDP port **Titan Center** binds for LAN host registration (host sends to this port).
pub const DEFAULT_CENTER_REGISTER_UDP_PORT: u16 = 7791;

/// Payload sent by `titan-host serve` so Titan Center can auto-add the device (LAN).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostAnnounceBeacon {
    pub kind: String,
    pub schema: u32,
    /// Control-plane TCP address clients should use, e.g. `192.168.1.10:7788`.
    pub host_control_addr: String,
    /// Display label in the center device list (may be empty; center will synthesize).
    pub label: String,
    /// OS-stable machine id from `titan-host` (`machine-uid`); empty for legacy beacons (schema v1).
    #[serde(default)]
    pub device_id: String,
}

impl HostAnnounceBeacon {
    #[must_use]
    pub fn new(
        host_control_addr: impl Into<String>,
        label: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Self {
        Self {
            kind: HOST_ANNOUNCE_BEACON_KIND.to_string(),
            schema: HOST_ANNOUNCE_SCHEMA_VERSION,
            host_control_addr: host_control_addr.into(),
            label: label.into(),
            device_id: device_id.into(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != HOST_ANNOUNCE_BEACON_KIND {
            return Err("host announce beacon kind mismatch");
        }
        if self.schema != 1 && self.schema != HOST_ANNOUNCE_SCHEMA_VERSION {
            return Err("host announce beacon schema mismatch");
        }
        if self.host_control_addr.trim().is_empty() {
            return Err("empty host_control_addr");
        }
        Ok(())
    }
}

/// JSON `kind` for [`CenterPollBeacon`]; hosts listen on [`DEFAULT_CENTER_POLL_UDP_PORT`] UDP.
pub const CENTER_POLL_BEACON_KIND: &str = "titan.v1.center_poll";

/// Schema for [`CenterPollBeacon`].
pub const CENTER_POLL_SCHEMA_VERSION: u32 = 1;

/// UDP **destination** port for center→LAN poll packets; `titan-host serve` binds this port to listen.
pub const DEFAULT_CENTER_POLL_UDP_PORT: u16 = 7792;

/// Center periodically broadcasts this so hosts reply with [`HostAnnounceBeacon`] to `register_udp_port`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CenterPollBeacon {
    pub kind: String,
    pub schema: u32,
    /// Send [`HostAnnounceBeacon`] to `(source_ip_of_this_packet, register_udp_port)`.
    pub register_udp_port: u16,
}

impl CenterPollBeacon {
    #[must_use]
    pub fn new(register_udp_port: u16) -> Self {
        Self {
            kind: CENTER_POLL_BEACON_KIND.to_string(),
            schema: CENTER_POLL_SCHEMA_VERSION,
            register_udp_port,
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
        Ok(())
    }
}

#[cfg(test)]
mod announce_tests {
    use super::*;

    #[test]
    fn host_announce_roundtrip() {
        let b = HostAnnounceBeacon::new("192.168.1.2:7788", "rack-a", "machine-id-hex");
        let v = serde_json::to_vec(&b).unwrap();
        let d: HostAnnounceBeacon = serde_json::from_slice(&v).unwrap();
        assert_eq!(d, b);
        d.validate().unwrap();
    }

    #[test]
    fn host_announce_schema_v1_without_device_id_still_parses() {
        let json = br#"{"kind":"titan.v1.host_announce","schema":1,"host_control_addr":"10.0.0.2:7788","label":"h1"}"#;
        let d: HostAnnounceBeacon = serde_json::from_slice(json).unwrap();
        assert!(d.device_id.is_empty());
        d.validate().unwrap();
    }

    #[test]
    fn center_poll_roundtrip() {
        let b = CenterPollBeacon::new(7791);
        let v = serde_json::to_vec(&b).unwrap();
        let d: CenterPollBeacon = serde_json::from_slice(&v).unwrap();
        assert_eq!(d, b);
        d.validate().unwrap();
    }
}
