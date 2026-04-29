//! Local JSON list of [`VmWindowRecord`] (same schema as Titan Center SQLite rows).

use std::path::PathBuf;

use titan_common::VmWindowRecord;

pub(crate) fn vm_windows_store_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|b| b.join("titan-host").join("vm_windows.json"))
}

pub(crate) fn load_vm_windows() -> Vec<VmWindowRecord> {
    let Some(p) = vm_windows_store_path() else {
        return Vec::new();
    };
    let Ok(s) = std::fs::read_to_string(&p) else {
        return Vec::new();
    };
    serde_json::from_str(&s).unwrap_or_default()
}

pub(crate) fn save_vm_windows(rows: &[VmWindowRecord]) -> std::io::Result<()> {
    let Some(p) = vm_windows_store_path() else {
        return Ok(());
    };
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(rows)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(&p, s)
}
