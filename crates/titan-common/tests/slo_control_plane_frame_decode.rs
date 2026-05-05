//! ## Latency SLO scope (center ↔ host, inner network)
//!
//! **Hard sub-path (measured here, no egui):** time from a complete on-wire control-plane host frame
//! in memory to a decoded [`titan_common::ControlHostFrame`] (parse header + payload +
//! `decode_control_host_payload`).
//! **Excluded:** TCP RTT, kernel scheduling, and full UI frame time (see product plan).
//!
//! CI runs tests in debug by default; the strict budget is asserted only in release builds.

use std::time::Instant;

use titan_common::{
    Capabilities, ControlHostFrame, ControlResponse, FRAME_HEADER_LEN, MAX_PAYLOAD_BYTES,
    decode_control_host_payload, encode_control_host_frame, parse_header,
};

fn read_control_host_frame_blocking(buf: &[u8]) -> ControlHostFrame {
    assert!(buf.len() >= FRAME_HEADER_LEN);
    let header_arr: [u8; FRAME_HEADER_LEN] = buf[..FRAME_HEADER_LEN]
        .try_into()
        .expect("FRAME_HEADER_LEN exact slice");
    let (_ver, len) = parse_header(&header_arr).expect("parse header");
    assert!(len <= MAX_PAYLOAD_BYTES);
    let payload = &buf[FRAME_HEADER_LEN..FRAME_HEADER_LEN + len as usize];
    decode_control_host_payload(payload).expect("decode payload")
}

#[test]
fn control_host_frame_decode_p99_budget() {
    let caps = Capabilities::default();
    let wire = encode_control_host_frame(&ControlHostFrame::Response {
        id: 1,
        body: ControlResponse::Pong { capabilities: caps },
    })
    .expect("encode control host frame");

    let n = 2000usize;
    let mut samples = Vec::with_capacity(n);
    for _ in 0..n {
        let t0 = Instant::now();
        let _f = read_control_host_frame_blocking(&wire);
        samples.push(t0.elapsed().as_nanos() as u64);
    }
    samples.sort_unstable();
    let p50 = samples[n / 2];
    let p99 = samples[(n * 99) / 100];
    eprintln!("control host frame decode p50={p50}ns p99={p99}ns (n={n})");

    #[cfg(not(debug_assertions))]
    assert!(
        p99 < 10_000_000,
        "p99 {p99}ns should stay under 10ms decode budget in release"
    );
}
