//! Selected-host control helpers (telemetry reader spawn).

use super::super::CenterApp;

impl CenterApp {
    /// Start (or keep) a telemetry QUIC reader for the selected device (same cap as [`Self::spawn_telemetry_reader_for`]).
    pub(crate) fn spawn_fleet_telemetry_selected(&mut self) {
        if self.endpoints.is_empty() {
            return;
        }
        let idx = self
            .selected_host
            .min(self.endpoints.len().saturating_sub(1));
        let ep = &self.endpoints[idx];
        let host_key = CenterApp::endpoint_addr_key(&ep.addr);
        self.spawn_telemetry_reader_for(host_key, ep.addr.clone());
    }
}
