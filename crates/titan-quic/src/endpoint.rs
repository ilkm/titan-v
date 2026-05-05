//! QUIC endpoint helpers (server + client) wired with `quinn` 0.11 + `rustls` 0.23 + ring.
//!
//! Centralises ALPN choice, transport defaults (idle timeout, keep-alive, BBR), and the
//! mTLS verifier wiring so call sites stay short.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use quinn::{ClientConfig, Endpoint, EndpointConfig, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::alpn;
use crate::identity::Identity;
use crate::pairing::Pairing;
use crate::trust_store::TrustStore;
use crate::verifier::{FingerprintClientVerifier, FingerprintServerVerifier};

/// Aggressive idle timeout so a dead peer is detected by quinn at ~500 ms even without an
/// application heartbeat. Application-layer 50 ms heartbeat (telemetry `HostHeartbeat`) typically
/// fires the offline path first; this is the transport-level fallback for crashed peers.
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_millis(500);
/// Must satisfy `keepalive < idle_timeout / 3` so quinn sends ≥3 PING attempts before
/// `max_idle_timeout` fires.
const DEFAULT_KEEPALIVE: Duration = Duration::from_millis(150);

pub fn install_default_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Builds a `quinn::ServerConfig` that requires mTLS and uses the fingerprint verifier.
pub fn build_server_config(
    identity: &Identity,
    trust: Arc<TrustStore>,
    pairing: Option<Arc<Pairing>>,
) -> Result<ServerConfig> {
    let cert = CertificateDer::from(identity.cert_der.clone());
    let key = pkcs8_key_der(identity)?;
    let verifier = Arc::new(FingerprintClientVerifier::new(trust, pairing));
    let mut tls = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(vec![cert], key)
        .context("rustls server with_single_cert")?;
    tls.alpn_protocols = alpn::alpn_protocols_server();
    let crypto =
        quinn::crypto::rustls::QuicServerConfig::try_from(tls).context("quinn server crypto")?;
    let mut sc = ServerConfig::with_crypto(Arc::new(crypto));
    sc.transport_config(Arc::new(default_transport()));
    Ok(sc)
}

/// Builds a `quinn::ClientConfig` for Center→Host control connections.
pub fn build_client_config(identity: &Identity, trust: Arc<TrustStore>) -> Result<ClientConfig> {
    let cert = CertificateDer::from(identity.cert_der.clone());
    let key = pkcs8_key_der(identity)?;
    let verifier = Arc::new(FingerprintServerVerifier::new(trust));
    let mut tls = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_client_auth_cert(vec![cert], key)
        .context("rustls client with_client_auth_cert")?;
    tls.alpn_protocols = alpn::alpn_protocols_client_control();
    let crypto =
        quinn::crypto::rustls::QuicClientConfig::try_from(tls).context("quinn client crypto")?;
    let mut cc = ClientConfig::new(Arc::new(crypto));
    cc.transport_config(Arc::new(default_transport()));
    Ok(cc)
}

fn pkcs8_key_der(identity: &Identity) -> Result<PrivateKeyDer<'static>> {
    Ok(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        identity.key_pkcs8_der.clone(),
    )))
}

fn default_transport() -> TransportConfig {
    let mut t = TransportConfig::default();
    t.max_idle_timeout(Some(
        DEFAULT_IDLE_TIMEOUT
            .try_into()
            .expect("idle timeout fits VarInt"),
    ));
    t.keep_alive_interval(Some(DEFAULT_KEEPALIVE));
    t
}

/// Binds a UDP socket and returns the matching `quinn::Endpoint` for a Host server.
pub fn bind_server_endpoint(bind: SocketAddr, server_cfg: ServerConfig) -> Result<Endpoint> {
    let socket = std::net::UdpSocket::bind(bind).with_context(|| format!("UDP bind {bind}"))?;
    let runtime =
        quinn::default_runtime().context("quinn: no compatible tokio runtime detected")?;
    let endpoint = Endpoint::new(EndpointConfig::default(), Some(server_cfg), socket, runtime)
        .context("quinn endpoint new")?;
    Ok(endpoint)
}

/// SNI server name used for outgoing connections; the verifier ignores it but rustls requires
/// a syntactically valid name. We use `titan-host-<device_id>` so that wireshark traces are
/// readable; the verifier compares fingerprints, not names.
#[must_use]
pub fn sni_for_host(device_id: &str) -> String {
    let safe: String = device_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!(
        "titan-host-{}",
        if safe.is_empty() {
            "unknown".into()
        } else {
            safe
        }
    )
}
