//! Blocking control-plane TCP exchange helpers (used from std::thread).

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};

use titan_common::{
    decode_control_host_payload, decode_telemetry_push_payload, encode_control_request_frame,
    parse_header, Capabilities, ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame,
    ControlResponse, FRAME_HEADER_LEN, TELEMETRY_MAX_PAYLOAD_BYTES,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::tcp_tune::tune_connected_stream;

static CONTROL_PLANE_REQ_ID: AtomicU64 = AtomicU64::new(1);

pub fn capabilities_summary(c: &Capabilities) -> String {
    let mut s = format!(
        "hyperv={} linux_virsh={} gpu={} stream={} vmbus_in={} hw_spoof={} guest_agent={} stream_precheck={} spoof_net={} spoof_cp={} spoof_proc={} k_ipc={} whv={} vhid={} nvn={} wrtc={} wd={}",
        c.hyperv,
        c.linux_virsh_inventory,
        c.gpu_partition,
        c.streaming,
        c.vmbus_input,
        c.hardware_spoof,
        c.guest_agent,
        c.streaming_precheck,
        c.hyperv_spoof_host.network_identity,
        c.hyperv_spoof_host.vm_checkpoint_policy,
        c.hyperv_spoof_host.vm_processor_count,
        c.kernel_driver_ipc,
        c.winhv_guest_memory,
        c.vmbus_hid,
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

/// Telemetry TCP address paired with control-plane command `host:port`.
pub fn telemetry_addr_for_control(control_addr: &str) -> anyhow::Result<String> {
    let addr: SocketAddr = control_addr.trim().parse()?;
    Ok(titan_common::control_plane_telemetry_addr(addr).to_string())
}

pub async fn exchange_one(addr: &str, req: &ControlRequest) -> anyhow::Result<ControlResponse> {
    let id = CONTROL_PLANE_REQ_ID.fetch_add(1, Ordering::Relaxed);
    let mut stream = TcpStream::connect(addr).await?;
    let _ = tune_connected_stream(&stream);
    let frame = encode_control_request_frame(&ControlRequestFrame {
        id,
        body: req.clone(),
    })?;
    stream.write_all(&frame).await?;
    loop {
        let mut hdr = [0u8; FRAME_HEADER_LEN];
        stream.read_exact(&mut hdr).await?;
        let (_, len) = parse_header(&hdr)?;
        let mut payload = vec![0u8; len as usize];
        stream.read_exact(&mut payload).await?;
        match decode_control_host_payload(&payload)? {
            ControlHostFrame::Response { id: rid, body } if rid == id => return Ok(body),
            ControlHostFrame::Response { id: rid, .. } => {
                anyhow::bail!("unexpected control-plane response id {rid} (expected {id})");
            }
            ControlHostFrame::Push(_) => continue,
        }
    }
}

/// One framed telemetry push from the telemetry TCP.
pub async fn read_telemetry_push(stream: &mut TcpStream) -> anyhow::Result<ControlPush> {
    let mut hdr = [0u8; FRAME_HEADER_LEN];
    stream.read_exact(&mut hdr).await?;
    let (_, len) = parse_header(&hdr)?;
    if len > TELEMETRY_MAX_PAYLOAD_BYTES {
        anyhow::bail!("telemetry payload length {len} exceeds max");
    }
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).await?;
    Ok(decode_telemetry_push_payload(&payload)?)
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
