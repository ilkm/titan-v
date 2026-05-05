use std::sync::Arc;

use titan_quic::{Pairing, Role, TrustStore, load_or_generate};

use crate::host_app::model::{HostSecurity, HostUiPersist};
use crate::serve::ServeSecurity;

pub(super) fn init_host_security() -> HostSecurity {
    let device_id = crate::host_device_id::host_device_id_string();
    let identity_dir = crate::host_paths::identity_dir();
    let trust_path = crate::host_paths::trust_store_path();
    let identity = match load_or_generate(&identity_dir, Role::Host, &device_id) {
        Ok(id) => Arc::new(id),
        Err(e) => panic!("titan-host: cannot load/generate mTLS identity: {e}"),
    };
    let trust = match TrustStore::open(trust_path) {
        Ok(t) => Arc::new(t),
        Err(e) => panic!("titan-host: cannot open trust store: {e}"),
    };
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
