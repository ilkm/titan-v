//! Per-VM SOCKS5 / proxy binding schema (Phase 6 design surface; WinDivert wiring comes later).

use serde::{Deserialize, Serialize};

/// One entry in a shared outbound proxy pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyPoolEntry {
    pub id: String,
    /// `host:port` for SOCKS5.
    pub socks5_endpoint: String,
}

/// Binds a VM identity (name or UUID string) to a pool entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmProxyBinding {
    pub vm_name: String,
    pub proxy_id: String,
}
