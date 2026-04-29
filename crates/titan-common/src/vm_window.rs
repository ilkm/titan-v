//! LAN UDP: host → center registration of a **window** (planned VM) row for SQLite + UI.

use serde::{Deserialize, Serialize};

/// JSON `kind` for [`VmWindowRegisterBeacon`].
pub const VM_WINDOW_REGISTER_BEACON_KIND: &str = "titan.v1.vm_window";

/// Schema for [`VmWindowRegisterBeacon`]; bump when fields change incompatibly.
pub const VM_WINDOW_REGISTER_SCHEMA_VERSION: u32 = 1;

/// One VM window row (persisted in Titan Center SQLite; echoed in Titan Host local JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmWindowRecord {
    pub record_id: String,
    pub device_id: String,
    pub host_control_addr: String,
    #[serde(default)]
    pub host_label: String,
    pub cpu_count: u32,
    pub memory_mib: u32,
    pub disk_mib: u32,
    pub vm_directory: String,
    pub created_at_unix_ms: i64,
}

/// UDP JSON payload: Titan Host → Titan Center (same listener port as host announce).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmWindowRegisterBeacon {
    pub kind: String,
    pub schema: u32,
    pub record: VmWindowRecord,
}

impl VmWindowRegisterBeacon {
    #[must_use]
    pub fn new(record: VmWindowRecord) -> Self {
        Self {
            kind: VM_WINDOW_REGISTER_BEACON_KIND.to_string(),
            schema: VM_WINDOW_REGISTER_SCHEMA_VERSION,
            record,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != VM_WINDOW_REGISTER_BEACON_KIND {
            return Err("vm window beacon kind mismatch");
        }
        if self.schema != VM_WINDOW_REGISTER_SCHEMA_VERSION {
            return Err("vm window beacon schema mismatch");
        }
        validate_vm_window_record(&self.record)
    }
}

fn validate_vm_window_record(r: &VmWindowRecord) -> Result<(), &'static str> {
    if r.record_id.trim().is_empty() {
        return Err("empty record_id");
    }
    if r.host_control_addr.trim().is_empty() {
        return Err("empty host_control_addr");
    }
    if r.vm_directory.trim().is_empty() {
        return Err("empty vm_directory");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_window_register_beacon_json_roundtrip() {
        let record = VmWindowRecord {
            record_id: "rid-1".into(),
            device_id: "dev-1".into(),
            host_control_addr: "192.168.1.10:7788".into(),
            host_label: "lab".into(),
            cpu_count: 2,
            memory_mib: 4096,
            disk_mib: 65536,
            vm_directory: r"C:\VMs\001".into(),
            created_at_unix_ms: 1_700_000_000_000,
        };
        let b = VmWindowRegisterBeacon::new(record.clone());
        let raw = serde_json::to_vec(&b).unwrap();
        let got: VmWindowRegisterBeacon = serde_json::from_slice(&raw).unwrap();
        assert_eq!(got.record, record);
        assert!(got.validate().is_ok());
    }
}
