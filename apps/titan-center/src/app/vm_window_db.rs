//! Center-local SQLite for `vm_window_records` (single source of truth for VM windows).
//!
//! Center owns CRUD; hosts only render the rows that Center pushes via
//! [`titan_common::ControlRequest::ApplyVmWindowSnapshot`]. Schema mirrors what host used to
//! carry, so the JSON wire format stays intact.

use std::path::Path;

use rusqlite::{Connection, params};
use titan_common::{VM_WINDOW_FOLDER_ID_MIN, VmWindowRecord};

use super::device_store;

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS vm_window_records (
    record_id TEXT NOT NULL PRIMARY KEY,
    device_id TEXT NOT NULL,
    host_control_addr TEXT NOT NULL,
    host_label TEXT NOT NULL DEFAULT '',
    cpu_count INTEGER NOT NULL,
    memory_mib INTEGER NOT NULL,
    disk_mib INTEGER NOT NULL,
    vm_directory TEXT NOT NULL,
    vm_id INTEGER NOT NULL DEFAULT 0,
    remark TEXT NOT NULL DEFAULT '',
    created_at_unix_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_vm_window_device ON vm_window_records(device_id);
"#;

/// Forward-compat ALTERs for DBs created before a column was introduced. Idempotent: each
/// branch first checks `pragma_table_info` so re-running on a fresh DB does nothing.
fn ensure_remark_column(conn: &Connection) -> rusqlite::Result<()> {
    let has_remark: bool = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('vm_window_records') WHERE name='remark'",
        [],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !has_remark {
        conn.execute(
            "ALTER TABLE vm_window_records ADD COLUMN remark TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    Ok(())
}

/// Shared center DB path (`titan-db.sqlite`) used by all center SQLite tables.
pub fn center_vm_window_db_path() -> std::path::PathBuf {
    device_store::registration_db_path()
}

fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        tracing::warn!("vm_window_db: create_dir_all {:?}: {e}", parent);
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(DDL)?;
    ensure_remark_column(&conn)?;
    Ok(conn)
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
        vm_id: row.get::<_, i64>(8)? as u32,
        remark: row.get(9)?,
        created_at_unix_ms: row.get(10)?,
    })
}

const SELECT_COLS: &str = "record_id, device_id, host_control_addr, host_label, cpu_count, \
     memory_mib, disk_mib, vm_directory, vm_id, remark, created_at_unix_ms";

pub fn list_all(path: &Path) -> rusqlite::Result<Vec<VmWindowRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let conn = open(path)?;
    let sql = format!(
        "SELECT {SELECT_COLS} FROM vm_window_records ORDER BY created_at_unix_ms ASC, vm_id ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_from_sql)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Returns `true` when another row for the same `device_id` uses `vm_id` (≥ min) or the same
/// trimmed `vm_directory` (paths are per-host; IDs may repeat across different machines).
pub fn conflicts_for(
    path: &Path,
    record_id: &str,
    device_id: &str,
    vm_id: u32,
    vm_directory: &str,
) -> rusqlite::Result<bool> {
    let rows = list_all(path)?;
    let dir = vm_directory.trim();
    let did = device_id.trim();
    Ok(rows.iter().any(|r| {
        if r.record_id == record_id {
            return false;
        }
        if r.device_id.trim() != did {
            return false;
        }
        let same_id = r.vm_id >= VM_WINDOW_FOLDER_ID_MIN
            && vm_id >= VM_WINDOW_FOLDER_ID_MIN
            && r.vm_id == vm_id;
        let same_dir = !dir.is_empty() && r.vm_directory.trim() == dir;
        same_id || same_dir
    }))
}

pub fn upsert(path: &Path, row: &VmWindowRecord) -> rusqlite::Result<()> {
    let conn = open(path)?;
    conn.execute(
        "INSERT OR REPLACE INTO vm_window_records \
         (record_id, device_id, host_control_addr, host_label, cpu_count, memory_mib, disk_mib, \
          vm_directory, vm_id, remark, created_at_unix_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            row.record_id,
            row.device_id,
            row.host_control_addr,
            row.host_label,
            row.cpu_count as i64,
            row.memory_mib as i64,
            row.disk_mib as i64,
            row.vm_directory,
            row.vm_id as i64,
            row.remark,
            row.created_at_unix_ms,
        ],
    )?;
    Ok(())
}

/// Returns the number of rows actually deleted (0 when missing).
pub fn delete_by_record_id(path: &Path, record_id: &str) -> rusqlite::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let conn = open(path)?;
    let n = conn.execute(
        "DELETE FROM vm_window_records WHERE record_id = ?1",
        params![record_id],
    )?;
    Ok(n)
}
