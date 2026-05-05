use std::path::Path;

use rusqlite::Connection;

const SCHEMA_KV: &str = r#"
CREATE TABLE IF NOT EXISTS app_kv (
    key TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS host_managed_config (
    device_id TEXT NOT NULL PRIMARY KEY,
    config_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
"#;

const SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS registered_devices (
    sort_order INTEGER NOT NULL,
    device_id TEXT NOT NULL PRIMARY KEY,
    label TEXT NOT NULL,
    addr TEXT NOT NULL,
    last_caps TEXT NOT NULL DEFAULT '',
    last_vm_count INTEGER NOT NULL DEFAULT 0,
    last_known_online INTEGER NOT NULL DEFAULT 0,
    remark TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_registered_devices_sort ON registered_devices(sort_order);
"#;

const MIGRATE_TO_DEVICE_ID_PK: &str = r#"
BEGIN;
CREATE TABLE registered_devices_new (
    sort_order INTEGER NOT NULL,
    device_id TEXT NOT NULL PRIMARY KEY,
    label TEXT NOT NULL,
    addr TEXT NOT NULL,
    last_caps TEXT NOT NULL DEFAULT '',
    last_vm_count INTEGER NOT NULL DEFAULT 0,
    last_known_online INTEGER NOT NULL DEFAULT 0,
    remark TEXT NOT NULL DEFAULT ''
);
INSERT INTO registered_devices_new
    (sort_order, device_id, label, addr, last_caps, last_vm_count, last_known_online, remark)
SELECT
    sort_order,
    device_id,
    label,
    addr,
    last_caps,
    last_vm_count,
    last_known_online,
    remark
FROM registered_devices;
DROP TABLE registered_devices;
ALTER TABLE registered_devices_new RENAME TO registered_devices;
CREATE INDEX IF NOT EXISTS idx_registered_devices_sort ON registered_devices(sort_order);
COMMIT;
"#;

pub(super) fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        tracing::warn!("device_store: create_dir_all {:?}: {e}", parent);
    }
    let conn = Connection::open(path)?;
    if !table_exists(&conn, "registered_devices")? {
        conn.execute_batch(SCHEMA_V2)?;
        ensure_kv_schema(&conn)?;
    } else {
        ensure_device_id_pk_schema(&conn)?;
        ensure_kv_schema(&conn)?;
    }
    Ok(conn)
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

fn registered_devices_column_names(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("PRAGMA table_info(registered_devices)")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect()
}

fn registered_devices_primary_key_column(conn: &Connection) -> rusqlite::Result<Option<String>> {
    let mut stmt = conn.prepare("PRAGMA table_info(registered_devices)")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let pk: i64 = row.get(5)?;
        Ok((name, pk))
    })?;
    for r in rows {
        let (name, pk) = r?;
        if pk == 1 {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

fn ensure_remark_column(conn: &Connection) -> rusqlite::Result<()> {
    let cols = registered_devices_column_names(conn)?;
    if cols.iter().any(|c| c == "remark") {
        return Ok(());
    }
    conn.execute(
        "ALTER TABLE registered_devices ADD COLUMN remark TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    Ok(())
}

fn add_device_id_column_when_missing(conn: &Connection) -> rusqlite::Result<()> {
    let cols = registered_devices_column_names(conn)?;
    if cols.iter().any(|c| c == "device_id") {
        return Ok(());
    }
    conn.execute(
        "ALTER TABLE registered_devices ADD COLUMN device_id TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    Ok(())
}

fn backfill_empty_device_ids(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE registered_devices SET device_id = ('legacy:' || addr) WHERE trim(device_id) = ''",
        [],
    )?;
    Ok(())
}

fn rebuild_registered_devices_device_id_pk(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(MIGRATE_TO_DEVICE_ID_PK)?;
    Ok(())
}

fn ensure_device_id_pk_schema(conn: &Connection) -> rusqlite::Result<()> {
    if !table_exists(conn, "registered_devices")? {
        conn.execute_batch(SCHEMA_V2)?;
        ensure_kv_schema(conn)?;
        return Ok(());
    }
    ensure_remark_column(conn)?;
    let pk = registered_devices_primary_key_column(conn)?;
    if pk.as_deref() == Some("device_id") {
        return Ok(());
    }
    add_device_id_column_when_missing(conn)?;
    backfill_empty_device_ids(conn)?;
    rebuild_registered_devices_device_id_pk(conn)
}

fn ensure_kv_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA_KV)?;
    Ok(())
}
