//! Shared QUIC + mTLS transport primitives for Titan-v Center / Host.
//!
//! The crate is intentionally narrow: it owns the bits that both Center and Host need
//! identical copies of (ALPN labels, identity files, fingerprint trust store, pairing flag,
//! verifier glue, frame I/O over `quinn` streams). Higher-level orchestration — connection
//! pooling on Center, request dispatch on Host — lives in the respective applications.

pub mod alpn;
pub mod asn1;
pub mod endpoint;
pub mod frame_io;
pub mod identity;
pub mod pairing;
pub mod trust_store;
pub mod verifier;

pub use alpn::{ALPN_CONTROL_V1, ALPN_TELEMETRY_V1};
pub use endpoint::{
    bind_server_endpoint, build_client_config, build_server_config,
    install_default_crypto_provider, sni_for_host,
};
pub use frame_io::{
    read_one_control_host, read_one_control_request, read_one_telemetry_push, write_control_host,
    write_control_request, write_telemetry_push,
};
pub use identity::{Identity, Role, load_or_generate, sha256_hex};
pub use pairing::{Pairing, PairingSnapshot};
pub use trust_store::{TrustEntry, TrustStore};
