//! Host-side **pairing window**: a time-limited mode that auto-trusts the next Center
//! client cert observed during a QUIC handshake.
//!
//! Lifecycle:
//! 1. Operator opens the Host pairing UI → calls [`Pairing::open`] with a TTL (default 5 min).
//! 2. Center initiates QUIC; rustls verifier consults [`Pairing::observe_unknown_peer`]
//!    *before* rejecting. If pairing is open, the unknown SPKI is added to the trust store
//!    and pairing closes (single-shot).
//! 3. If TTL elapses without a connection, [`Pairing::is_open`] returns `false` and
//!    further unknown peers are rejected.
//!
//! State is process-local; pairing must be re-opened after Host restart. This is by design:
//! a stale pairing window persisted on disk would be a quiet supply-chain risk.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

use crate::trust_store::{TrustEntry, TrustStore};

#[derive(Debug, Clone, Copy, Default)]
pub struct PairingSnapshot {
    pub open: bool,
    pub ttl_remaining_ms: u64,
}

#[derive(Debug)]
struct State {
    deadline: Option<Instant>,
}

/// Single-shot pairing flag tied to a [`TrustStore`].
pub struct Pairing {
    inner: Mutex<State>,
    trust: Arc<TrustStore>,
}

impl std::fmt::Debug for Pairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pairing")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl Pairing {
    #[must_use]
    pub fn new(trust: Arc<TrustStore>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(State { deadline: None }),
            trust,
        })
    }

    pub fn open(&self, ttl: Duration) {
        let mut g = self.inner.lock();
        g.deadline = Some(Instant::now() + ttl);
    }

    pub fn close(&self) {
        let mut g = self.inner.lock();
        g.deadline = None;
    }

    #[must_use]
    pub fn snapshot(&self) -> PairingSnapshot {
        let g = self.inner.lock();
        match g.deadline {
            Some(d) => {
                let now = Instant::now();
                if d <= now {
                    PairingSnapshot {
                        open: false,
                        ttl_remaining_ms: 0,
                    }
                } else {
                    PairingSnapshot {
                        open: true,
                        ttl_remaining_ms: (d - now).as_millis() as u64,
                    }
                }
            }
            None => PairingSnapshot::default(),
        }
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.snapshot().open
    }

    /// Called from the Host's mTLS verifier the first time it sees an unknown peer cert.
    /// Returns `true` iff pairing was open and the peer was added to the trust store.
    pub fn observe_unknown_peer(
        &self,
        spki_sha256_hex: &str,
        peer_role: &str,
        peer_label: &str,
    ) -> bool {
        let mut g = self.inner.lock();
        let open_now = matches!(g.deadline, Some(d) if d > Instant::now());
        if !open_now {
            return false;
        }
        let added = self
            .trust
            .upsert(TrustEntry {
                fingerprint: spki_sha256_hex.to_string(),
                label: peer_label.to_string(),
                role: peer_role.to_string(),
                source: "pairing-window".to_string(),
                added_at_epoch_s: epoch_seconds(),
            })
            .map_err(|e| tracing::warn!(error = %e, "trust store upsert failed during pairing"))
            .is_ok();
        if added {
            g.deadline = None;
        }
        added
    }
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
