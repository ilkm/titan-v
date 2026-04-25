//! Host UI persistence shape (serde) shared by egui and control-plane apply.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use titan_common::{
    VmProvisionPlan, DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT,
};

use crate::config::VmGroup;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBindingRow {
    pub vm_name: String,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostUiPersist {
    pub listen: String,
    pub announce_enabled: bool,
    pub announce_periodic_secs: Option<u64>,
    pub center_register_udp_port: u16,
    pub center_poll_listen_port: u16,
    pub public_addr_override: String,
    pub label_override: String,
    pub agent_rows: Vec<AgentBindingRow>,
    pub batch_timeout_secs: u64,
    pub batch_fail_fast: bool,
    pub batch_vm: Vec<VmProvisionPlan>,
    pub batch_vm_group: Vec<VmGroup>,
}

impl Default for HostUiPersist {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:7788".into(),
            announce_enabled: true,
            announce_periodic_secs: None,
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            public_addr_override: String::new(),
            label_override: String::new(),
            agent_rows: Vec::new(),
            batch_timeout_secs: 600,
            batch_fail_fast: false,
            batch_vm: Vec::new(),
            batch_vm_group: Vec::new(),
        }
    }
}

impl HostUiPersist {
    pub fn parse_listen(&self) -> Result<SocketAddr, String> {
        self.listen
            .trim()
            .parse()
            .map_err(|e| format!("监听地址无效: {e}"))
    }

    /// Validates fields needed before enqueueing a remote config apply (no `serve` types).
    pub fn validate_for_remote_apply(&self) -> Result<(), String> {
        self.parse_listen()?;
        for row in &self.agent_rows {
            let vm = row.vm_name.trim();
            if vm.is_empty() {
                continue;
            }
            row.addr
                .trim()
                .parse::<SocketAddr>()
                .map_err(|e| format!("{} 的地址无效: {e}", row.vm_name))?;
        }
        Ok(())
    }
}
