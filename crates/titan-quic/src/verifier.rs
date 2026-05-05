//! Custom rustls verifiers that authenticate peers by SHA-256(SPKI) instead of CA chains.
//!
//! Both Center→Host and Host→Center handshakes use the same fingerprint-trust model:
//! * Compute `spki_sha256_hex` of the leaf cert SPKI.
//! * If it is in the [`TrustStore`], accept.
//! * Otherwise: on the **server** side, consult the optional [`Pairing`] (auto-trust if open);
//!   on the **client** side, reject hard so the UI can prompt TOFU.
//!
//! The crypto signature of the handshake is still validated via rustls's stock signature
//! verification (`WebPkiSupportedAlgorithms`); we only replace chain/identity validation.

use std::sync::Arc;

use rustls::DigitallySignedStruct;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::WebPkiSupportedAlgorithms;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};

use crate::identity::sha256_hex;
use crate::pairing::Pairing;
use crate::trust_store::TrustStore;

#[derive(Debug)]
pub struct FingerprintServerVerifier {
    trust: Arc<TrustStore>,
    sigalgs: WebPkiSupportedAlgorithms,
}

impl FingerprintServerVerifier {
    #[must_use]
    pub fn new(trust: Arc<TrustStore>) -> Self {
        Self {
            trust,
            sigalgs: rustls::crypto::ring::default_provider().signature_verification_algorithms,
        }
    }
}

impl ServerCertVerifier for FingerprintServerVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let fp = spki_fp_from_leaf(end_entity)?;
        if self.trust.contains(&fp) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(format!(
                "untrusted host certificate (fingerprint sha256:{fp})"
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.sigalgs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.sigalgs)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.sigalgs.supported_schemes()
    }
}

#[derive(Debug)]
pub struct FingerprintClientVerifier {
    trust: Arc<TrustStore>,
    pairing: Option<Arc<Pairing>>,
    sigalgs: WebPkiSupportedAlgorithms,
}

impl FingerprintClientVerifier {
    #[must_use]
    pub fn new(trust: Arc<TrustStore>, pairing: Option<Arc<Pairing>>) -> Self {
        Self {
            trust,
            pairing,
            sigalgs: rustls::crypto::ring::default_provider().signature_verification_algorithms,
        }
    }
}

impl ClientCertVerifier for FingerprintClientVerifier {
    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        let fp = spki_fp_from_leaf(end_entity)?;
        if self.trust.contains(&fp) {
            return Ok(ClientCertVerified::assertion());
        }
        if let Some(p) = self.pairing.as_ref()
            && p.observe_unknown_peer(&fp, "center", "")
        {
            return Ok(ClientCertVerified::assertion());
        }
        Err(rustls::Error::General(format!(
            "untrusted center certificate (fingerprint sha256:{fp})"
        )))
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.sigalgs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.sigalgs)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.sigalgs.supported_schemes()
    }
}

/// Re-exposed: parse leaf cert DER and return SHA-256(SPKI) hex.
pub fn spki_sha256_hex_for_leaf(cert: &CertificateDer<'_>) -> Result<String, rustls::Error> {
    spki_fp_from_leaf(cert)
}

fn spki_fp_from_leaf(cert: &CertificateDer<'_>) -> Result<String, rustls::Error> {
    let spki = crate::asn1::extract_spki_der(cert.as_ref())
        .map_err(|e| rustls::Error::General(format!("asn1 spki: {e}")))?;
    Ok(sha256_hex(&spki))
}
