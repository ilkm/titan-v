//! OS-stable machine id (`machine-uid`): LAN beacons and [`titan_common::Capabilities::device_id`].

/// Same value as sent in [`titan_common::HostAnnounceBeacon::device_id`].
pub fn host_device_id_string() -> String {
    match machine_uid::get() {
        Ok(s) => {
            let t = s.trim();
            if t.is_empty() {
                tracing::warn!("machine_uid returned empty string");
                fallback_device_id()
            } else {
                t.to_string()
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "machine_uid::get failed");
            fallback_device_id()
        }
    }
}

fn fallback_device_id() -> String {
    format!(
        "fallback:{}",
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".into())
    )
}
