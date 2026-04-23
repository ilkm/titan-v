//! Integration: control plane Ping / Pong over TCP.

use std::time::Duration;

use titan_common::{
    encode_request_frame, parse_header, read_response_frame, ControlRequest, ControlResponse,
};
use titan_host::serve::{handle_connection, ServeState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn ping_pong_over_tcp() {
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
    let frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    client.write_all(&frame).await.unwrap();

    let mut buf = Vec::new();
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    buf.extend_from_slice(&hdr);
    buf.extend_from_slice(&payload);

    let res = read_response_frame(&mut buf.as_slice()).unwrap();
    match res {
        ControlResponse::Pong { capabilities } => {
            assert!(!capabilities.gpu_partition);
        }
        ControlResponse::ServerError { .. } => panic!("unexpected ServerError"),
        _ => panic!("unexpected response variant"),
    }

    drop(client);
    server.await.unwrap();
}
