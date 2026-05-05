//! On-disk locations for `titan-center` private state (mTLS identity, trust store).

use std::path::PathBuf;

const ENV_IDENTITY_DIR: &str = "TITAN_CENTER_IDENTITY_DIR";
const ENV_TRUST_DB: &str = "TITAN_CENTER_TRUST_DB_PATH";

#[must_use]
pub fn identity_dir() -> PathBuf {
    if let Ok(s) = std::env::var(ENV_IDENTITY_DIR)
        && !s.trim().is_empty()
    {
        return PathBuf::from(s);
    }
    default_state_dir().join("identity")
}

#[must_use]
pub fn trust_store_path() -> PathBuf {
    if let Ok(s) = std::env::var(ENV_TRUST_DB)
        && !s.trim().is_empty()
    {
        return PathBuf::from(s);
    }
    default_state_dir().join("trusted_hosts.json")
}

fn default_state_dir() -> PathBuf {
    if let Some(d) = dirs::data_local_dir() {
        return d.join("titan-center");
    }
    if let Some(h) = dirs::home_dir() {
        return h.join(".titan-center");
    }
    PathBuf::from("./titan-center")
}
