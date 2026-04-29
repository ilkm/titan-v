//! Integration: ApplySpoofProfile over control-plane TCP (host spoof / OpenVMM path not wired).

use std::time::Duration;

use titan_common::{
    ControlHostFrame, ControlRequest, ControlResponse, VmSpoofProfile, encode_request_frame,
    parse_header, read_control_host_frame,
};
use titan_host::serve::{ServeState, handle_connection};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn read_one_control_response(client: &mut tokio::net::TcpStream) -> ControlResponse {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    let mut buf = Vec::new();
    buf.extend_from_slice(&hdr);
    buf.extend_from_slice(&payload);
    match read_control_host_frame(&mut buf.as_slice()).unwrap() {
        ControlHostFrame::Response { body, .. } => body,
        other => panic!("unexpected control host frame: {other:?}"),
    }
}

fn assert_apply_spoof_removed(res: ControlResponse) {
    match res {
        ControlResponse::ServerError { code, message } => {
            assert_eq!(code, 501);
            assert!(
                message.contains("removed") || message.contains("mother_image"),
                "{message}"
            );
        }
        other => panic!("expected ServerError 501: {other:?}"),
    }
}

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

    let res = read_one_control_response(&mut client).await;
    assert_apply_spoof_removed(res);

    drop(client);
    server.await.unwrap();
}
