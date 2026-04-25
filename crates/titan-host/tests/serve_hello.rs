//! Integration: Hello / HelloAck over TCP.

use std::time::Duration;

use titan_common::{
    encode_request_frame, parse_header, read_control_host_frame, ControlHostFrame, ControlRequest,
    ControlResponse,
};
use titan_host::serve::{handle_connection, ServeState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

async fn spawn_hello_server() -> (std::net::SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let st = ServeState::for_test();
        handle_connection(sock, Duration::from_secs(5), 1, st)
            .await
            .unwrap();
    });
    (addr, server)
}

async fn assert_hello_ack(client: &mut TcpStream) {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    let mut buf = Vec::new();
    buf.extend_from_slice(&hdr);
    buf.extend_from_slice(&payload);
    let res = match read_control_host_frame(&mut buf.as_slice()).unwrap() {
        ControlHostFrame::Response { body, .. } => body,
        other => panic!("unexpected control host frame: {other:?}"),
    };
    match res {
        ControlResponse::HelloAck { .. } => {}
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn hello_hello_ack_over_tcp() {
    let (addr, server) = spawn_hello_server().await;
    let mut client = TcpStream::connect(addr).await.unwrap();
    let frame = encode_request_frame(&ControlRequest::Hello).unwrap();
    client.write_all(&frame).await.unwrap();
    assert_hello_ack(&mut client).await;
    drop(client);
    server.await.unwrap();
}
