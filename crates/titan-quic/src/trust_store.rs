//! SHA-256(SPKI) fingerprint trust store, persisted as JSON next to other Titan state.
//!
//! No SQL dependency: trust tables are tiny (typically O(10–100) peers) and we want both
//! Center and Host to read/write them without pulling rusqlite. The on-disk shape is
//! `{"version": 1, "entries": [{...}]}` so we can grow fields later.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustEntry {
    /// SHA-256(SPKI) hex (lowercase, 64 chars).
    pub fingerprint: String,
    /// Optional opaque label (`device_id` for Host entries; free-form for Center entries).
    #[serde(default)]
    pub label: String,
    /// `host` or `center`; record-only, mTLS validation does not consult this.
    pub role: String,
    /// `manual` (TOFU prompt / paired window), `auto` (LAN beacon match), or `cli`.
    pub source: String,
    /// Unix epoch seconds; informational.
    pub added_at_epoch_s: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TrustFile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    entries: Vec<TrustEntry>,
}

const fn default_version() -> u32 {
    1
}

#[derive(Debug)]
pub struct TrustStore {
    path: PathBuf,
    state: Mutex<BTreeMap<String, TrustEntry>>,
}

impl TrustStore {
    pub fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create trust dir {}", parent.display()))?;
        }
        let initial = if path.exists() {
            read_file(&path)?
        } else {
            TrustFile::default()
        };
        let mut map = BTreeMap::new();
        for e in initial.entries {
            if e.fingerprint.len() == 64 {
                map.insert(e.fingerprint.clone(), e);
            }
        }
        Ok(Self {
            path,
            state: Mutex::new(map),
        })
    }

    pub fn list(&self) -> Vec<TrustEntry> {
        self.state
            .lock()
            .expect("trust mutex")
            .values()
            .cloned()
            .collect()
    }

    pub fn contains(&self, fingerprint: &str) -> bool {
        let g = self.state.lock().expect("trust mutex");
        g.contains_key(fingerprint)
    }

    /// Inserts or refreshes an entry; persists synchronously. Returns `true` if newly added.
    pub fn upsert(&self, entry: TrustEntry) -> Result<bool> {
        if entry.fingerprint.len() != 64 {
            return Err(anyhow::anyhow!("trust: invalid fingerprint length"));
        }
        let mut g = self.state.lock().expect("trust mutex");
        let added = !g.contains_key(&entry.fingerprint);
        g.insert(entry.fingerprint.clone(), entry);
        write_file(
            &self.path,
            &TrustFile {
                version: default_version(),
                entries: g.values().cloned().collect(),
            },
        )?;
        Ok(added)
    }

    pub fn remove(&self, fingerprint: &str) -> Result<bool> {
        let mut g = self.state.lock().expect("trust mutex");
        let removed = g.remove(fingerprint).is_some();
        if removed {
            write_file(
                &self.path,
                &TrustFile {
                    version: default_version(),
                    entries: g.values().cloned().collect(),
                },
            )?;
        }
        Ok(removed)
    }
}

fn read_file(path: &Path) -> Result<TrustFile> {
    let bytes = fs::read(path).with_context(|| format!("read trust {}", path.display()))?;
    if bytes.is_empty() {
        return Ok(TrustFile::default());
    }
    let parsed: TrustFile = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse trust {}", path.display()))?;
    Ok(parsed)
}

fn write_file(path: &Path, file: &TrustFile) -> Result<()> {
    let body = serde_json::to_vec_pretty(file).context("serialize trust json")?;
    let mut tmp = path.to_path_buf();
    let mut name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    tmp.set_file_name(name);
    fs::write(&tmp, &body).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("rename to {}", path.display()))?;
    Ok(())
}
