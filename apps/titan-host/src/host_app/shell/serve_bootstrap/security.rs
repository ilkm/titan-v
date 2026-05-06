use std::sync::Arc;

use titan_quic::{Pairing, Role, TrustStore, load_or_generate};

use crate::host_app::model::{HostSecurity, HostUiPersist};
use crate::serve::ServeSecurity;

pub(super) fn init_host_security() -> HostSecurity {
    let device_id = crate::host_device_id::host_device_id_string();
    let identity_dir = crate::host_paths::identity_dir();
    let trust_path = crate::host_paths::trust_store_path();
    let identity = open_host_identity(&identity_dir, &device_id);
    let trust = open_host_trust_store(&trust_path);
    let pairing = Pairing::new(trust.clone());
    auto_open_pairing_if_first_run(&trust, &pairing);
    tracing::info!(
        device_id = %device_id,
        fingerprint = %identity.spki_sha256_hex,
        "host mTLS identity ready"
    );
    HostSecurity {
        identity,
        trust,
        pairing,
    }
}

fn open_host_identity(
    identity_dir: &std::path::Path,
    device_id: &str,
) -> Arc<titan_quic::Identity> {
    if let Ok(identity) = load_or_generate(identity_dir, Role::Host, device_id) {
        return Arc::new(identity);
    }
    let fallback_dir = std::env::temp_dir().join("titan-host-fallback-identity");
    match load_or_generate(&fallback_dir, Role::Host, device_id) {
        Ok(identity) => {
            tracing::error!(
                primary = %identity_dir.display(),
                fallback = %fallback_dir.display(),
                "host mTLS identity fallback to temp dir"
            );
            Arc::new(identity)
        }
        Err(e) => {
            tracing::error!(
                primary = %identity_dir.display(),
                fallback = %fallback_dir.display(),
                error = %e,
                "host mTLS identity init failed"
            );
            std::process::exit(2);
        }
    }
}

fn open_host_trust_store(trust_path: &std::path::Path) -> Arc<TrustStore> {
    if let Ok(store) = TrustStore::open(trust_path.to_path_buf()) {
        return Arc::new(store);
    }
    let fallback = std::env::temp_dir().join("titan-host-fallback-trusted-peers.sqlite");
    match TrustStore::open(fallback.clone()) {
        Ok(store) => {
            tracing::error!(
                primary = %trust_path.display(),
                fallback = %fallback.display(),
                "host trust store fallback to temp path"
            );
            Arc::new(store)
        }
        Err(e) => {
            tracing::error!(
                primary = %trust_path.display(),
                fallback = %fallback.display(),
                error = %e,
                "host trust store init failed"
            );
            std::process::exit(2);
        }
    }
}

pub(super) fn build_serve_security(
    persist: &HostUiPersist,
    host_security: &HostSecurity,
) -> ServeSecurity {
    let _ = persist;
    ServeSecurity {
        identity: host_security.identity.clone(),
        trust: host_security.trust.clone(),
        pairing: host_security.pairing.clone(),
    }
}

fn auto_open_pairing_if_first_run(trust: &Arc<TrustStore>, pairing: &Arc<Pairing>) {
    if !trust.list().is_empty() {
        return;
    }
    pairing.open(std::time::Duration::from_secs(5 * 60));
    tracing::info!(
        "host mTLS: empty trust store; auto-opened pairing window (5 min) for first Center"
    );
}
