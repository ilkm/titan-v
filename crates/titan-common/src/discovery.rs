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
    /// `titan-host serve` M2 TCP address, e.g. `192.168.1.10:7788`.
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
