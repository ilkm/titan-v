//! Per-endpoint live state (VM list, volumes, telemetry) for fleet / multi-host UI.

use std::time::Instant;

use titan_common::{DiskVolume, HostResourceStats, VmBrief};

#[derive(Clone, Debug, Default)]
pub struct HostLiveState {
    pub vms: Vec<VmBrief>,
    pub volumes: Vec<DiskVolume>,
    pub telemetry_live: bool,
    pub last_telemetry_at: Option<Instant>,
    pub last_resource_live: Option<HostResourceStats>,
}

impl HostLiveState {
    pub fn clear_telemetry(&mut self) {
        self.telemetry_live = false;
        self.last_telemetry_at = None;
        self.last_resource_live = None;
    }
}
