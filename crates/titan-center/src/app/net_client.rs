//! Blocking M2 TCP exchange helpers (used from std::thread).

use titan_common::{
    decode_response_payload, encode_request_frame, parse_header, Capabilities, ControlRequest,
    ControlResponse,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub fn capabilities_summary(c: &Capabilities) -> String {
    format!(
        "hyperv={} gpu={} stream={} vmbus_in={} hw_spoof={} guest_agent={} stream_precheck={} spoof_net={} spoof_cp={} spoof_proc={} k_ipc={} whv={} vhid={} nvn={} wrtc={} wd={}",
        c.hyperv,
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
    )
}

pub async fn exchange_one(addr: &str, req: &ControlRequest) -> anyhow::Result<ControlResponse> {
    let mut stream = TcpStream::connect(addr).await?;
    let frame = encode_request_frame(req)?;
    stream.write_all(&frame).await?;
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    stream.read_exact(&mut hdr).await?;
    let (_, len) = parse_header(&hdr)?;
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).await?;
    Ok(decode_response_payload(&payload)?)
}

pub async fn hello_host(addr: &str) -> anyhow::Result<ControlResponse> {
    exchange_one(addr, &ControlRequest::Hello).await
}

pub async fn ping_host(addr: &str) -> anyhow::Result<ControlResponse> {
    exchange_one(addr, &ControlRequest::Ping).await
}
