//! WinDivert + per-VM SOCKS5 binding (Phase 6).
//!
//! WinDivert itself is not linked; this module loads **proxy pool + binding** rows from TOML for
//! validation and future integration.

use std::path::Path;

use serde::Deserialize;
use titan_common::{ProxyPoolEntry, VmProxyBinding};

/// Documents intended binding: one SOCKS entry from the pool per [`VmProxyBinding`].
#[must_use]
pub fn describe_proxy_binding_schema() -> &'static str {
    "SOCKS5 pool entries map to VM vNIC/MAC policy; WinDivert user-mode path is not linked."
}

/// Validates pool + binding rows (no kernel / socket I/O).
pub fn validate_proxy_config(
    pool: &[ProxyPoolEntry],
    binds: &[VmProxyBinding],
) -> Result<(), String> {
    for b in binds {
        if !pool.iter().any(|p| p.id == b.proxy_id) {
            return Err(format!("unknown proxy_id {}", b.proxy_id));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ProxyBindingsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    pool: Vec<ProxyPoolEntry>,
    #[serde(default)]
    binding: Vec<VmProxyBinding>,
}

/// Loads `pool` + `binding` tables from a TOML file (see unit test for shape).
pub fn load_proxy_bindings_file(
    path: &Path,
) -> anyhow::Result<(Vec<ProxyPoolEntry>, Vec<VmProxyBinding>)> {
    let raw = std::fs::read_to_string(path)?;
    let file: ProxyBindingsFile = toml::from_str(&raw)?;
    if file.schema_version > 1 {
        anyhow::bail!(
            "unsupported proxy-bindings schema_version {} (max 1)",
            file.schema_version
        );
    }
    Ok((file.pool, file.binding))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_and_validate_proxy_toml() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
schema_version = 1
[[pool]]
id = "p1"
socks5_endpoint = "127.0.0.1:1080"

[[binding]]
vm_name = "vm-a"
proxy_id = "p1"
"#
        )
        .unwrap();
        let (pool, binds) = load_proxy_bindings_file(f.path()).unwrap();
        validate_proxy_config(&pool, &binds).unwrap();
    }
}
