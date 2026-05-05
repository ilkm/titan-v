use std::sync::Arc;

use crate::app::CenterSecurity;

pub(super) fn init_center_security() -> CenterSecurity {
    use titan_quic::{Role, TrustStore, load_or_generate};

    let identity_dir = crate::app::center_paths::identity_dir();
    let trust_path = crate::app::center_paths::trust_store_path();
    let device_id = device_id_for_center();
    let identity = match load_or_generate(&identity_dir, Role::Center, &device_id) {
        Ok(id) => Arc::new(id),
        Err(e) => panic!("titan-center: cannot load/generate mTLS identity: {e}"),
    };
    let trust = match TrustStore::open(trust_path) {
        Ok(t) => Arc::new(t),
        Err(e) => panic!("titan-center: cannot open trust store: {e}"),
    };
    titan_quic::install_default_crypto_provider();
    if let Err(e) = crate::app::net::init_global(identity.clone(), trust.clone()) {
        panic!("titan-center: cannot init QUIC client: {e}");
    }
    tracing::info!(
        device_id = %device_id,
        fingerprint = %identity.spki_sha256_hex,
        "center mTLS identity ready"
    );
    CenterSecurity { identity, trust }
}

fn device_id_for_center() -> String {
    machine_uid::get().unwrap_or_else(|_| "unknown-center".to_string())
}
