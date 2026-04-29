//! Experimental QUIC fleet control plane (UDP, port offset from command TCP — see `titan_common::control_plane_quic_addr`).
//!
//! Uses a short-lived self-signed certificate (localhost). Center clients should use insecure
//! verification until a shared CA is deployed.

use std::net::SocketAddr;
use std::sync::Arc;

use quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

fn fleet_quic_server_config() -> Result<ServerConfig, String> {
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".into()]).map_err(|e| e.to_string())?;
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let mut rustls_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], priv_key.into())
        .map_err(|e| e.to_string())?;
    rustls_cfg.alpn_protocols = vec![b"titan-fleet-v1".to_vec()];
    let crypto = Arc::new(QuicServerConfig::try_from(rustls_cfg).map_err(|e| e.to_string())?);
    let mut server_config = ServerConfig::with_crypto(crypto);
    let transport = Arc::get_mut(&mut server_config.transport)
        .ok_or_else(|| "fleet QUIC: transport config".to_string())?;
    transport.max_concurrent_uni_streams(0u32.into());
    Ok(server_config)
}

/// Listens for QUIC connections on `bind` until the process exits (runs in a background task).
pub fn spawn_fleet_quic_listener(bind: SocketAddr) {
    tokio::spawn(async move {
        let cfg = match fleet_quic_server_config() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "fleet QUIC: server config failed");
                return;
            }
        };
        let endpoint = match Endpoint::server(cfg, bind) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, %bind, "fleet QUIC: bind failed");
                return;
            }
        };
        tracing::info!(%bind, "fleet QUIC listening (UDP)");
        loop {
            let Some(incoming) = endpoint.accept().await else {
                break;
            };
            match incoming.await {
                Ok(conn) => {
                    tracing::info!(peer = %conn.remote_address(), "fleet QUIC peer connected");
                }
                Err(e) => tracing::debug!(error = %e, "fleet QUIC accept failed"),
            }
        }
    });
}
