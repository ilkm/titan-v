//! Parse `agent-bindings.toml` for [`titan_vmm::hyperv::HypervHostRuntime`].

use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use titan_vmm::hyperv::AgentBindingTable;

#[derive(Debug, Deserialize)]
struct AgentBindingsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    binding: Vec<BindingRow>,
}

#[derive(Debug, Deserialize)]
struct BindingRow {
    vm_name: String,
    addr: String,
    /// Parsed for forward compatibility; not validated in v1.
    #[serde(default)]
    #[allow(dead_code)]
    psk_sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentBindingsFileSer {
    schema_version: u32,
    binding: Vec<BindingRowSer>,
}

#[derive(Debug, Serialize)]
struct BindingRowSer {
    vm_name: String,
    addr: String,
}

/// Loads bindings from TOML. Empty file or missing `binding` yields an empty map.
pub fn load_agent_bindings(path: &Path) -> anyhow::Result<AgentBindingTable> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read agent bindings {}", path.display()))?;
    let file: AgentBindingsFile = toml::from_str(&raw)
        .with_context(|| format!("parse agent bindings TOML ({})", path.display()))?;
    if file.schema_version > 1 {
        anyhow::bail!(
            "unsupported agent-bindings schema_version {} (max 1)",
            file.schema_version
        );
    }
    let out = AgentBindingTable::new();
    for row in file.binding {
        let vm = row.vm_name.trim().to_string();
        if vm.is_empty() {
            continue;
        }
        let addr: SocketAddr = row
            .addr
            .trim()
            .parse()
            .with_context(|| format!("invalid addr for vm {vm}: {}", row.addr))?;
        out.insert(vm, addr);
    }
    Ok(out)
}

/// Writes the current binding table to TOML (schema v1), sorted by `vm_name`.
pub fn save_agent_bindings(path: &Path, table: &AgentBindingTable) -> anyhow::Result<()> {
    let mut rows: Vec<(String, SocketAddr)> = table
        .iter()
        .map(|e| (e.key().clone(), *e.value()))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let file = AgentBindingsFileSer {
        schema_version: 1,
        binding: rows
            .into_iter()
            .map(|(vm, addr)| BindingRowSer {
                vm_name: vm,
                addr: addr.to_string(),
            })
            .collect(),
    };
    let raw = toml::to_string_pretty(&file).context("serialize agent bindings TOML")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create_dir_all {}", parent.display()))?;
    }
    let mut f = fs::File::create(path)
        .with_context(|| format!("write agent bindings {}", path.display()))?;
    f.write_all(raw.as_bytes())
        .with_context(|| format!("write agent bindings {}", path.display()))?;
    Ok(())
}

/// Builds a map from optional path; missing path → empty map.
///
/// If a path is given but the file is missing, unreadable, or invalid TOML, returns an **empty**
/// table and a human-readable notice for [`titan_common::Capabilities::host_notice`] instead of
/// failing startup.
pub fn load_or_empty(path: Option<&Path>) -> (AgentBindingTable, Option<String>) {
    match path {
        None => (AgentBindingTable::new(), None),
        Some(p) => match load_agent_bindings(p) {
            Ok(t) => (t, None),
            Err(e) => {
                let msg = format!(
                    "agent-bindings: path missing or system file unreadable [{}] ({})",
                    p.display(),
                    e
                );
                tracing::warn!(
                    path = %p.display(),
                    error = %e,
                    "agent-bindings load failed; serving with empty bindings and reporting host_notice"
                );
                (AgentBindingTable::new(), Some(msg))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_two_rows() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
schema_version = 1
[[binding]]
vm_name = "a"
addr = "127.0.0.1:9001"
[[binding]]
vm_name = "b"
addr = "127.0.0.1:9002"
"#
        )
        .unwrap();
        let m = load_agent_bindings(f.path()).unwrap();
        assert_eq!(m.len(), 2);
        assert!(m.contains_key("a"));
    }

    #[test]
    fn load_or_empty_missing_file_still_yields_empty_table_and_notice() {
        let p = Path::new("/nonexistent/titan-agent-bindings-99.toml");
        let (t, w) = load_or_empty(Some(p));
        assert!(t.is_empty());
        let msg = w.expect("notice");
        assert!(msg.contains("path missing"));
        assert!(msg.contains("nonexistent"));
    }

    #[test]
    fn save_roundtrip() {
        let f = NamedTempFile::new().unwrap();
        let table = AgentBindingTable::new();
        table.insert("z".into(), "127.0.0.1:1".parse().unwrap());
        table.insert("a".into(), "127.0.0.1:2".parse().unwrap());
        save_agent_bindings(f.path(), &table).unwrap();
        let m = load_agent_bindings(f.path()).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(
            *m.get("a").unwrap().value(),
            "127.0.0.1:2".parse::<SocketAddr>().unwrap()
        );
    }
}
