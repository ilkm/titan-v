//! Multi-frame session: two Ping requests on one TCP connection.

use std::time::Duration;

use titan_common::{
    decode_response_payload, encode_request_frame, parse_header, ControlRequest, ControlResponse,
};
use titan_host::serve::ServeState;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn read_one_response(client: &mut tokio::net::TcpStream) -> ControlResponse {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    client.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    client.read_exact(&mut payload).await.unwrap();
    decode_response_payload(&payload).unwrap()
}

#[tokio::test]
async fn two_pings_one_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let st = ServeState::for_test();
        titan_host::serve::handle_connection(sock, Duration::from_secs(10), 42, st)
            .await
            .unwrap();
    });

    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    for _ in 0..2 {
        let frame = encode_request_frame(&ControlRequest::Ping).unwrap();
        client.write_all(&frame).await.unwrap();
        let res = read_one_response(&mut client).await;
        match res {
            ControlResponse::Pong { .. } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }
    drop(client);
    server.await.unwrap();
}
