//! Integration: RegisterGuestAgent updates bindings; Ping reports guest_agent capability.

use std::time::Duration;

use titan_common::{
    encode_request_frame, parse_header, read_response_frame, ControlRequest, ControlResponse,
};
use titan_host::serve::{handle_connection, ServeState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn read_one_res(client: &mut tokio::net::TcpStream) -> ControlResponse {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    let mut buf = hdr.to_vec();
    buf.extend_from_slice(&payload);
    read_response_frame(&mut buf.as_slice()).unwrap()
}

#[tokio::test]
async fn register_guest_agent_then_ping_shows_agent_capability() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let st = ServeState::for_test();
        handle_connection(sock, Duration::from_secs(5), 1, st)
            .await
            .unwrap();
    });

    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();

    let frame = encode_request_frame(&ControlRequest::RegisterGuestAgent {
        vm_name: "demo-vm".into(),
        guest_agent_addr: "127.0.0.1:9001".into(),
    })
    .unwrap();
    client.write_all(&frame).await.unwrap();
    let res = read_one_res(&mut client).await;
    match res {
        ControlResponse::GuestAgentRegisterAck { vm_name } => assert_eq!(vm_name, "demo-vm"),
        other => panic!("unexpected {other:?}"),
    }

    let frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    client.write_all(&frame).await.unwrap();
    let res = read_one_res(&mut client).await;
    match res {
        ControlResponse::Pong { capabilities } => {
            assert!(
                capabilities.guest_agent,
                "expected guest_agent after register"
            );
        }
        other => panic!("unexpected {other:?}"),
    }

    drop(client);
    server.await.unwrap();
}
