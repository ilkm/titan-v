//! Messages from the background network thread to the UI thread.

use titan_common::{ControlPush, HostResourceStats, VmBrief};

pub enum NetUiMsg {
    Caps {
        summary: String,
    },
    VmInventory(Vec<VmBrief>),
    /// Result of background Hello used by the manual add-host dialog (online check + `device_id`).
    AddHostVerifyDone {
        /// Must match [`CenterApp::add_host_verify_session`] or the result is ignored (cancel / watchdog).
        session_id: u64,
        addr: String,
        ok: bool,
        device_id: String,
        caps_summary: String,
        error: String,
    },
    /// LAN UDP: host announced its QUIC control-plane address + mTLS SPKI fingerprint.
    HostAnnounced {
        quic_addr: String,
        label: String,
        /// OS machine id from host (`machine-uid`); never empty in v3 beacons.
        device_id: String,
        /// SHA-256(SPKI) hex (lowercase, 64 chars). Center auto-trusts on first sight.
        fingerprint: String,
    },
    /// JPEG desktop frame for device management preview (`control_addr` = normalized host address key).
    DesktopSnapshot {
        control_addr: String,
        jpeg_bytes: Vec<u8>,
    },
    /// Host CPU / memory / NIC rates from [`ControlRequest::HostResourceSnapshot`] (same poll as desktop preview).
    HostResources {
        control_addr: String,
        stats: HostResourceStats,
    },
    /// Background desktop fetch batch finished; clears busy flag.
    DesktopFetchCycleDone,
    /// Result of a periodic `Hello` probe (`control_addr` = normalized address key).
    HostReachability {
        control_addr: String,
        online: bool,
    },
    /// Background reachability probe batch finished; clears busy flag.
    ReachabilityProbeCycleDone,
    /// Telemetry TCP: framed `ControlPush` (VM/disk `HostTelemetry` or periodic `HostResourceLive`).
    /// `host_key` is [`CenterApp::endpoint_addr_key`] for the host that opened this stream.
    /// `session_gen` must match the active telemetry session for that stream or the message is stale.
    HostTelemetry {
        host_key: String,
        session_gen: u64,
        push: ControlPush,
    },
    /// Telemetry TCP read failed or stream ended for `host_key` / `session_gen`.
    TelemetryLinkLost {
        host_key: String,
        session_gen: u64,
    },
    /// One host result from a fleet fan-out operation (`spawn_fleet_exchange`).
    FleetOpResult {
        host_key: String,
        ok: bool,
        detail: String,
    },
    /// Fleet fan-out worker finished (clears [`CenterApp::fleet_busy`]).
    FleetOpDone,
    /// Background `ApplyHostUiPersistJson` push to one host finished.
    HostUiPushDone {
        ok: bool,
        detail: String,
    },
    Error(String),
}
