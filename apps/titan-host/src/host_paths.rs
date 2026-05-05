//! On-disk locations for `titan-host` private state (mTLS identity, trust store).
//!
//! Each helper honours an env override so integration tests can use a `tempfile::TempDir`
//! without polluting the developer's home directory.

use std::path::PathBuf;

const ENV_IDENTITY_DIR: &str = "TITAN_HOST_IDENTITY_DIR";
const ENV_TRUST_DB: &str = "TITAN_HOST_TRUST_DB_PATH";

/// Directory holding `identity.cert.pem` and `identity.key.pem`.
#[must_use]
pub fn identity_dir() -> PathBuf {
    if let Ok(s) = std::env::var(ENV_IDENTITY_DIR)
        && !s.trim().is_empty()
    {
        return PathBuf::from(s);
    }
    default_state_dir().join("identity")
}

/// JSON file backing the trusted-Center fingerprint store.
#[must_use]
pub fn trust_store_path() -> PathBuf {
    if let Ok(s) = std::env::var(ENV_TRUST_DB)
        && !s.trim().is_empty()
    {
        return PathBuf::from(s);
    }
    default_state_dir().join("trusted_centers.json")
}

fn default_state_dir() -> PathBuf {
    if let Some(d) = dirs::data_local_dir() {
        return d.join("titan-host");
    }
    if let Some(h) = dirs::home_dir() {
        return h.join(".titan-host");
    }
    PathBuf::from("./titan-host")
}
