//! Convenience wrappers around [`crate::app::net::quic_client::exchange_one`].
//!
//! Exists so call sites read like the original TCP client (`hello_host(addr)` etc.) without
//! having to know about the connection cache or fingerprint trust path.

use titan_common::{Capabilities, ControlRequest, ControlResponse};

use super::quic_client::exchange_one;

pub fn capabilities_summary(c: &Capabilities) -> String {
    let mut s = format!(
        "openvmm={} gpu={} stream={} hw_spoof={} guest_agent={} stream_precheck={} spoof_net={} spoof_cp={} spoof_proc={} k_ipc={} whv={} nvn={} wrtc={} wd={}",
        c.openvmm,
        c.gpu_partition,
        c.streaming,
        c.hardware_spoof,
        c.guest_agent,
        c.streaming_precheck,
        c.host_spoof_probes.network_identity,
        c.host_spoof_probes.vm_checkpoint_policy,
        c.host_spoof_probes.vm_processor_count,
        c.kernel_driver_ipc,
        c.winhv_guest_memory,
        c.streaming_nvenc,
        c.streaming_webrtc,
        c.windivert_forward,
    );
    if !c.host_notice.is_empty() {
        use std::fmt::Write;
        let _ = write!(s, " | {}", c.host_notice);
    }
    s
}

pub async fn hello_host(addr: &str) -> anyhow::Result<ControlResponse> {
    exchange_one(addr, &ControlRequest::Hello).await
}

pub async fn fetch_desktop_snapshot(addr: &str) -> anyhow::Result<ControlResponse> {
    exchange_one(
        addr,
        &ControlRequest::HostDesktopSnapshot {
            max_width: 1280,
            max_height: 720,
            jpeg_quality: 55,
        },
    )
    .await
}

pub async fn fetch_host_resource_snapshot(addr: &str) -> anyhow::Result<ControlResponse> {
    exchange_one(addr, &ControlRequest::HostResourceSnapshot).await
}
