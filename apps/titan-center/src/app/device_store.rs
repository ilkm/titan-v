//! SQLite store for the registered host/device list (control-plane TCP addresses).
//!
//! File location: [`registration_db_path`] → `{dirs::data_local_dir()}/titan-center/devices.sqlite`
//! (e.g. macOS `~/Library/Application Support/...`, Linux `~/.local/share/...`), **not** the git repo root.
//! Settings JSON lives in `app_kv` under key [`super::constants::PERSIST_KEY`]; devices in `registered_devices`.

mod schema;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::PERSIST_KEY;
use super::persist_data::HostEndpoint;

/// User-local SQLite path (alongside other `titan-center` app data).
pub fn registration_db_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join("titan-center").join("devices.sqlite")
}

pub(crate) fn open(path: &Path) -> rusqlite::Result<Connection> {
    schema::open(path)
}

/// Load ordered device rows. Missing DB file yields an empty list.
pub fn load_registered_devices(path: &Path) -> rusqlite::Result<Vec<HostEndpoint>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let conn = open(path)?;
    let mut stmt = conn.prepare(
        "SELECT label, addr, device_id, remark, last_caps, last_vm_count, last_known_online \
         FROM registered_devices ORDER BY sort_order ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(HostEndpoint {
            label: row.get(0)?,
            addr: row.get(1)?,
            device_id: row.get(2)?,
            remark: row.get(3)?,
            last_caps: row.get(4)?,
            last_vm_count: row.get::<_, i64>(5)? as u32,
            last_known_online: row.get::<_, i64>(6)? != 0,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Replace the full device list (transactional).
pub fn save_registered_devices(path: &Path, devices: &[HostEndpoint]) -> rusqlite::Result<()> {
    let mut conn = open(path)?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM registered_devices", [])?;
    for (i, mut d) in devices.iter().cloned().enumerate() {
        d.ensure_device_id();
        tx.execute(
            "INSERT INTO registered_devices \
             (sort_order, device_id, label, addr, remark, last_caps, last_vm_count, last_known_online) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                i as i64,
                d.device_id,
                d.label,
                d.addr,
                &d.remark,
                &d.last_caps,
                d.last_vm_count as i64,
                i64::from(d.last_known_online),
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Reads `endpoints` from a legacy `titan_center_state_v1` JSON blob (before SQLite split).
/// Loads serialized [`super::persist_data::CenterPersist`] JSON from SQLite (canonical store).
pub fn load_center_persist_json(path: &Path) -> rusqlite::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = open(path)?;
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM app_kv WHERE key = ?1",
            [PERSIST_KEY],
            |row| row.get(0),
        )
        .optional()?;
    Ok(v)
}

/// Persists center UI settings JSON (same key as legacy eframe `PERSIST_KEY`).
pub fn save_center_persist_json(path: &Path, json: &str) -> rusqlite::Result<()> {
    let conn = open(path)?;
    conn.execute(
        "INSERT INTO app_kv (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![PERSIST_KEY, json],
    )?;
    Ok(())
}

/// Saves a host UI JSON draft (same schema as Titan Host `titan_host_ui_v1`) for push from Center or CLI.
pub fn upsert_host_managed_config(
    path: &Path,
    device_id: &str,
    json: &str,
) -> rusqlite::Result<()> {
    let conn = open(path)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO host_managed_config (device_id, config_json, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(device_id) DO UPDATE SET config_json = excluded.config_json, \
         updated_at = excluded.updated_at",
        params![device_id, json, now],
    )?;
    Ok(())
}

pub fn load_host_managed_config(path: &Path, device_id: &str) -> rusqlite::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = open(path)?;
    let v: Option<String> = conn
        .query_row(
            "SELECT config_json FROM host_managed_config WHERE device_id = ?1",
            [device_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(v)
}

pub fn addr_for_device_id(path: &Path, device_id: &str) -> rusqlite::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = open(path)?;
    let v: Option<String> = conn
        .query_row(
            "SELECT addr FROM registered_devices WHERE device_id = ?1",
            [device_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(v)
}

pub fn legacy_endpoints_from_center_json(json: &str) -> Option<Vec<HostEndpoint>> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let arr = v.get("endpoints")?.as_array()?;
    if arr.is_empty() {
        return None;
    }
    serde_json::from_value(serde_json::Value::Array(arr.clone())).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_two_host_endpoints() -> Vec<HostEndpoint> {
        vec![
            HostEndpoint {
                label: "a".into(),
                addr: "127.0.0.1:1".into(),
                device_id: "id-a".into(),
                remark: "lab".into(),
                last_caps: "caps".into(),
                last_vm_count: 3,
                last_known_online: true,
            },
            HostEndpoint {
                label: "b".into(),
                addr: "127.0.0.1:2".into(),
                device_id: "id-b".into(),
                remark: String::new(),
                last_caps: String::new(),
                last_vm_count: 0,
                last_known_online: false,
            },
        ]
    }

    fn assert_host_endpoints_equal(got: &[HostEndpoint], devs: &[HostEndpoint]) {
        assert_eq!(got.len(), devs.len());
        for (a, b) in got.iter().zip(devs.iter()) {
            assert_eq!(a.label, b.label);
            assert_eq!(a.addr, b.addr);
            assert_eq!(a.device_id, b.device_id);
            assert_eq!(a.remark, b.remark);
            assert_eq!(a.last_caps, b.last_caps);
            assert_eq!(a.last_vm_count, b.last_vm_count);
            assert_eq!(a.last_known_online, b.last_known_online);
        }
    }

    #[test]
    fn roundtrip_registered_devices() {
        let mut tmp = std::env::temp_dir();
        tmp.push("titan-center-test-devices.sqlite");
        let _ = std::fs::remove_file(&tmp);
        let devs = sample_two_host_endpoints();
        save_registered_devices(&tmp, &devs).unwrap();
        let got = load_registered_devices(&tmp).unwrap();
        assert_host_endpoints_equal(&got, &devs);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn save_fills_empty_device_id_with_legacy_prefix() {
        let mut tmp = std::env::temp_dir();
        tmp.push("titan-center-test-devices-legacyid.sqlite");
        let _ = std::fs::remove_file(&tmp);
        let mut d = HostEndpoint {
            label: "m".into(),
            addr: "10.0.0.5:7788".into(),
            device_id: String::new(),
            remark: String::new(),
            last_caps: String::new(),
            last_vm_count: 0,
            last_known_online: false,
        };
        save_registered_devices(&tmp, std::slice::from_ref(&d)).unwrap();
        let got = load_registered_devices(&tmp).unwrap();
        assert_eq!(got[0].device_id, "legacy:10.0.0.5:7788");
        d.ensure_device_id();
        assert_eq!(d.device_id, "legacy:10.0.0.5:7788");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn legacy_json_parses_endpoints() {
        let json = r#"{"endpoints":[{"label":"x","addr":"10.0.0.1:7788","last_caps":"","last_vm_count":0,"last_known_online":false}],"accounts":[]}"#;
        let eps = legacy_endpoints_from_center_json(json).unwrap();
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].addr, "10.0.0.1:7788");
        assert!(eps[0].device_id.is_empty());
    }
}
