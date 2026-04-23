//! Offline guest identity tooling (Phase 2B): VHDX mount + hive edits + Sysprep orchestration.
//!
//! Default builds expose only [`offline_spoof_status`]. Enable the `offline-hive` feature when
//! implementing platform-specific mounting (Windows-only; requires admin and explicit operator
//! approval — see repository `docs/hyperv-secure-boot-matrix.md`).

/// Human-readable build profile for operators and CI.
#[must_use]
pub fn offline_spoof_status() -> &'static str {
    #[cfg(feature = "offline-hive")]
    {
        "titan-offline-spoof: offline-hive feature enabled (implementation may still be partial)."
    }
    #[cfg(not(feature = "offline-hive"))]
    {
        "titan-offline-spoof: offline-hive feature disabled; no VHDX/hive mutations are compiled."
    }
}

#[cfg(all(test, not(feature = "offline-hive")))]
mod tests {
    use super::*;

    #[test]
    fn default_status_mentions_disabled() {
        assert!(offline_spoof_status().contains("disabled"));
    }
}
