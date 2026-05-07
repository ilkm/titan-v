//! Trust-on-first-use coordinator: detect untrusted-cert handshake errors raised by the
//! [`titan_quic::FingerprintServerVerifier`] and surface a one-shot prompt to the operator.
//!
//! Glue only — the actual mTLS validation lives in `titan_quic`; this module just lifts the
//! fingerprint out of the rustls error string and converts a confirmation into a trust-store
//! upsert + automatic re-verification.

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use titan_quic::TrustEntry;

use crate::app::CenterApp;
use crate::app::TofuPrompt;

/// Substring emitted by [`titan_quic::FingerprintServerVerifier`] for unknown server certs.
const UNTRUSTED_HOST_PREFIX: &str = "untrusted host certificate (fingerprint sha256:";

impl CenterApp {
    /// Parses the fingerprint hex from an mTLS error string, if it matches the verifier's wording.
    #[must_use]
    pub(crate) fn extract_untrusted_host_fingerprint(error: &str) -> Option<String> {
        let start = error.find(UNTRUSTED_HOST_PREFIX)? + UNTRUSTED_HOST_PREFIX.len();
        let tail = error.get(start..)?;
        let end = tail.find(')')?;
        let hex = tail.get(..end)?.trim();
        if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            Some(hex.to_ascii_lowercase())
        } else {
            None
        }
    }

    /// Returns true if a TOFU prompt was raised; caller should suppress the offline toast.
    pub(crate) fn maybe_raise_tofu_for_verify_error(&mut self, addr: &str, error: &str) -> bool {
        let Some(fp) = Self::extract_untrusted_host_fingerprint(error) else {
            return false;
        };
        if self.center_security.trust.contains(&fp) {
            return false;
        }
        self.tofu_pending = Some(TofuPrompt {
            host_addr: addr.to_string(),
            fingerprint: fp,
            label: addr.to_string(),
        });
        self.ctx.request_repaint();
        true
    }

    /// Operator confirmed the TOFU prompt → upsert into trust store + re-trigger Hello probe.
    pub(crate) fn confirm_tofu_pending(&mut self) {
        let Some(prompt) = self.tofu_pending.take() else {
            return;
        };
        if let Err(e) = self
            .center_security
            .trust
            .upsert(trust_entry_for_tofu(&prompt))
        {
            tracing::warn!(error = %e, "trust store upsert failed during TOFU");
            self.last_net_error = format!("trust store: {e}");
            return;
        }
        self.retry_add_host_after_tofu(&prompt.host_addr);
    }

    fn retry_add_host_after_tofu(&mut self, host_addr: &str) {
        crate::app::net::forget_host(host_addr);
        self.add_host_dialog_open = true;
        self.add_host_dialog_ip = host_addr.split(':').next().unwrap_or("").to_string();
        self.add_host_dialog_port = host_addr.rsplit(':').next().unwrap_or("").to_string();
        self.spawn_add_host_verify(host_addr.to_string());
        self.ctx.request_repaint();
    }

    pub(crate) fn dismiss_tofu_pending(&mut self) {
        self.tofu_pending = None;
        self.ctx.request_repaint();
    }
}

fn trust_entry_for_tofu(prompt: &TofuPrompt) -> TrustEntry {
    TrustEntry {
        fingerprint: prompt.fingerprint.clone(),
        label: prompt.label.clone(),
        role: "host".into(),
        source: "tofu-manual".into(),
        added_at_epoch_s: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fp_from_quinn_error_string() {
        let s = concat!(
            "quinn handshake 192.0.2.1:7788: connection lost: tls: ",
            "untrusted host certificate (fingerprint sha256:",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef)"
        );
        let fp = CenterApp::extract_untrusted_host_fingerprint(s);
        assert_eq!(
            fp.as_deref(),
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        );
    }

    #[test]
    fn returns_none_for_unrelated_errors() {
        let s = "quinn handshake 192.0.2.1:7788: timed out";
        assert!(CenterApp::extract_untrusted_host_fingerprint(s).is_none());
    }

    #[test]
    fn rejects_short_hex() {
        let s = "untrusted host certificate (fingerprint sha256:deadbeef)";
        assert!(CenterApp::extract_untrusted_host_fingerprint(s).is_none());
    }
}
