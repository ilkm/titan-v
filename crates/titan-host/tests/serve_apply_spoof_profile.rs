//! Integration: ApplySpoofProfile over M2 TCP.

use std::time::Duration;

use titan_common::{
    encode_request_frame, parse_header, read_response_frame, ControlRequest, ControlResponse,
    VmSpoofProfile,
};
use titan_host::serve::{handle_connection, ServeState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn apply_spoof_profile_dry_run_roundtrip() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let st = ServeState::for_test();
        handle_connection(sock, Duration::from_secs(8), 1, st)
            .await
            .unwrap();
    });

    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    let spoof = VmSpoofProfile {
        dynamic_mac: true,
        disable_checkpoints: false,
        ..Default::default()
    };
    let frame = encode_request_frame(&ControlRequest::ApplySpoofProfile {
        vm_name: "nonexistent-vm".into(),
        dry_run: true,
        spoof,
    })
    .unwrap();
    client.write_all(&frame).await.unwrap();

    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    let mut buf = Vec::new();
    buf.extend_from_slice(&hdr);
    buf.extend_from_slice(&payload);

    let res = read_response_frame(&mut buf.as_slice()).unwrap();
    #[cfg(not(windows))]
    match res {
        ControlResponse::ServerError { code, message } => {
            assert_eq!(code, 500);
            assert!(
                message.contains("Windows") || message.contains("Hyper-V"),
                "{message}"
            );
        }
        other => panic!("expected ServerError on non-Windows: {other:?}"),
    }
    #[cfg(windows)]
    match res {
        ControlResponse::SpoofApplyAck { dry_run, .. } => assert!(dry_run),
        ControlResponse::ServerError { .. } => {}
        other => panic!("unexpected response: {other:?}"),
    }

    drop(client);
    server.await.unwrap();
}
