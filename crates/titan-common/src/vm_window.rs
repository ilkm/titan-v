//! Shared `VmWindowRecord` shape and helpers (host SQLite is the single source of truth).

use serde::{Deserialize, Serialize};

/// Lower bound for [`VmWindowRecord::vm_id`] when used as `{vm_root}/{vm_id}` folder id (non-zero).
pub const VM_WINDOW_FOLDER_ID_MIN: u32 = 100;
/// Upper bound for [`VmWindowRecord::vm_id`] as folder id (inclusive).
pub const VM_WINDOW_FOLDER_ID_MAX: u32 = 999_999_999;

/// Smallest id in [`VM_WINDOW_FOLDER_ID_MIN`..=`VM_WINDOW_FOLDER_ID_MAX`] not present in `existing`.
///
/// `vm_id == 0` entries in storage are ignored (legacy / unspecified). If every id in range is
/// taken, returns [`VM_WINDOW_FOLDER_ID_MAX`] (caller should still run duplicate checks).
pub fn next_unused_vm_folder_id(existing: impl IntoIterator<Item = u32>) -> u32 {
    let mut ids: Vec<u32> = existing
        .into_iter()
        .filter(|&id| (VM_WINDOW_FOLDER_ID_MIN..=VM_WINDOW_FOLDER_ID_MAX).contains(&id))
        .collect();
    if ids.is_empty() {
        return VM_WINDOW_FOLDER_ID_MIN;
    }
    ids.sort_unstable();
    ids.dedup();
    let mut expect = VM_WINDOW_FOLDER_ID_MIN;
    for &id in &ids {
        if id < expect {
            continue;
        }
        if id > expect {
            return expect;
        }
        expect = match expect.checked_add(1) {
            Some(n) if n <= VM_WINDOW_FOLDER_ID_MAX => n,
            _ => return VM_WINDOW_FOLDER_ID_MAX,
        };
    }
    if expect <= VM_WINDOW_FOLDER_ID_MAX {
        expect
    } else {
        VM_WINDOW_FOLDER_ID_MAX
    }
}

/// One VM window row (persisted in Titan Host SQLite `vm_window_records`; center mirrors via TCP RPC + telemetry push).
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
    /// Numeric VM folder id when using `{vm_root}/{vm_id}` layout; `0` = legacy / unspecified.
    #[serde(default)]
    pub vm_id: u32,
    /// Free-form user note shown on the window-management card; edited via the Center UI.
    #[serde(default)]
    pub remark: String,
    pub created_at_unix_ms: i64,
}

/// Validates a row before persistence or control-plane apply.
pub fn validate_vm_window_record(r: &VmWindowRecord) -> Result<(), &'static str> {
    if r.record_id.trim().is_empty() {
        return Err("empty record_id");
    }
    if r.host_control_addr.trim().is_empty() {
        return Err("empty host_control_addr");
    }
    if r.vm_directory.trim().is_empty() {
        return Err("empty vm_directory");
    }
    if r.vm_id != 0 && !(VM_WINDOW_FOLDER_ID_MIN..=VM_WINDOW_FOLDER_ID_MAX).contains(&r.vm_id) {
        return Err("vm_id out of range");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_unused_vm_folder_id_gaps() {
        assert_eq!(next_unused_vm_folder_id([100]), 101);
        assert_eq!(next_unused_vm_folder_id([100, 101, 200]), 102);
        assert_eq!(next_unused_vm_folder_id(100_u32..=200_u32), 201);
        assert_eq!(
            next_unused_vm_folder_id([] as [u32; 0]),
            VM_WINDOW_FOLDER_ID_MIN
        );
        assert_eq!(next_unused_vm_folder_id([0, 50]), VM_WINDOW_FOLDER_ID_MIN);
        assert_eq!(next_unused_vm_folder_id([150]), VM_WINDOW_FOLDER_ID_MIN);
    }
}
