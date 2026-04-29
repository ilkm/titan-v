//! SQLite table `vm_window_records` (host-reported planned VM windows).

use std::path::Path;

use rusqlite::{Connection, params};
use titan_common::VmWindowRecord;

const VM_WINDOWS_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS vm_window_records (
    record_id TEXT NOT NULL PRIMARY KEY,
    device_id TEXT NOT NULL,
    host_control_addr TEXT NOT NULL,
    host_label TEXT NOT NULL DEFAULT '',
    cpu_count INTEGER NOT NULL,
    memory_mib INTEGER NOT NULL,
    disk_mib INTEGER NOT NULL,
    vm_directory TEXT NOT NULL,
    created_at_unix_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_vm_window_device ON vm_window_records(device_id);
CREATE INDEX IF NOT EXISTS idx_vm_window_addr ON vm_window_records(host_control_addr);
"#;

pub(super) fn ensure_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(VM_WINDOWS_DDL)?;
    Ok(())
}

fn row_from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<VmWindowRecord> {
    Ok(VmWindowRecord {
        record_id: row.get(0)?,
        device_id: row.get(1)?,
        host_control_addr: row.get(2)?,
        host_label: row.get(3)?,
        cpu_count: row.get::<_, i64>(4)? as u32,
        memory_mib: row.get::<_, i64>(5)? as u32,
        disk_mib: row.get::<_, i64>(6)? as u32,
        vm_directory: row.get(7)?,
        created_at_unix_ms: row.get(8)?,
    })
}

pub fn load_vm_windows(path: &Path) -> rusqlite::Result<Vec<VmWindowRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let conn = crate::app::device_store::open(path)?;
    let mut stmt = conn.prepare(
        "SELECT record_id, device_id, host_control_addr, host_label, cpu_count, \
         memory_mib, disk_mib, vm_directory, created_at_unix_ms \
         FROM vm_window_records ORDER BY created_at_unix_ms ASC",
    )?;
    let rows = stmt.query_map([], row_from_sql)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn insert_vm_window(path: &Path, row: &VmWindowRecord) -> rusqlite::Result<()> {
    let conn = crate::app::device_store::open(path)?;
    conn.execute(
        "INSERT OR REPLACE INTO vm_window_records \
         (record_id, device_id, host_control_addr, host_label, cpu_count, memory_mib, disk_mib, \
          vm_directory, created_at_unix_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            row.record_id,
            row.device_id,
            row.host_control_addr,
            row.host_label,
            row.cpu_count as i64,
            row.memory_mib as i64,
            row.disk_mib as i64,
            row.vm_directory,
            row.created_at_unix_ms,
        ],
    )?;
    Ok(())
}
