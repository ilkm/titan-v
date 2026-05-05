//! Integration: Center pushes [`ControlRequest::ApplyVmWindowSnapshot`]; the host applies it
//! to its in-memory list (`VmWindowReloadMsg::Replace`) and acks.

use std::sync::Arc;
use std::sync::mpsc as sync_mpsc;
use std::time::Duration;

use titan_common::{
    ControlHostFrame, ControlRequest, ControlResponse, VmWindowRecord, encode_request_frame,
    parse_header, read_control_host_frame,
};
use titan_host::serve::{ServeState, VmWindowReloadMsg, handle_connection};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

fn sample_record(record_id: &str, vm_id: u32) -> VmWindowRecord {
    VmWindowRecord {
        record_id: record_id.to_string(),
        device_id: "test-host-device".to_string(),
        host_control_addr: "127.0.0.1:7788".to_string(),
        host_label: "test-host".to_string(),
        cpu_count: 4,
        memory_mib: 8192,
        disk_mib: 65_536,
        vm_directory: format!("vm-{vm_id}"),
        vm_id,
        created_at_unix_ms: 1_700_000_000_000,
    }
}

fn spawn_serve(state: Arc<ServeState>) -> (std::net::SocketAddr, JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        handle_connection(sock, Duration::from_secs(10), 1, state)
            .await
            .unwrap();
    });
    (addr, server)
}

async fn read_one_response(client: &mut TcpStream) -> ControlResponse {
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

async fn send_apply_snapshot(
    client: &mut TcpStream,
    device_id: &str,
    rows: &[VmWindowRecord],
) -> ControlResponse {
    let records_json = serde_json::to_string(rows).unwrap();
    let req = ControlRequest::ApplyVmWindowSnapshot {
        device_id: device_id.to_string(),
        records_json,
    };
    let frame = encode_request_frame(&req).unwrap();
    client.write_all(&frame).await.unwrap();
    read_one_response(client).await
}

fn build_state_with_reload_tx() -> (Arc<ServeState>, sync_mpsc::Receiver<VmWindowReloadMsg>) {
    let (tx, rx) = sync_mpsc::channel();
    let state = ServeState::for_test_with_reload_tx(tx);
    (state, rx)
}

fn assert_replace_msg(rx: &sync_mpsc::Receiver<VmWindowReloadMsg>, expected: &[VmWindowRecord]) {
    let msg = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("VmWindowReloadMsg::Replace should be delivered");
    let VmWindowReloadMsg::Replace { records } = msg;
    assert_eq!(records.len(), expected.len());
    for (a, b) in records.iter().zip(expected.iter()) {
        assert_eq!(a.record_id, b.record_id);
        assert_eq!(a.device_id, b.device_id);
        assert_eq!(a.vm_id, b.vm_id);
    }
}

#[tokio::test]
async fn apply_snapshot_with_matching_device_id_replaces_and_acks() {
    let (state, rx) = build_state_with_reload_tx();
    let (addr, server) = spawn_serve(state);
    let mut client = TcpStream::connect(addr).await.unwrap();

    let rows = vec![sample_record("r-1", 100), sample_record("r-2", 101)];
    let res = send_apply_snapshot(&mut client, "", &rows).await;
    match res {
        ControlResponse::ApplyVmWindowSnapshotAck { ok, applied, .. } => {
            assert!(ok, "ack should be ok=true on empty device_id (broadcast)");
            assert_eq!(applied, 2);
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_replace_msg(&rx, &rows);

    drop(client);
    server.await.unwrap();
}

#[tokio::test]
async fn apply_snapshot_with_mismatched_device_id_is_rejected() {
    let (state, rx) = build_state_with_reload_tx();
    let (addr, server) = spawn_serve(state);
    let mut client = TcpStream::connect(addr).await.unwrap();

    let rows = vec![sample_record("r-1", 100)];
    let res = send_apply_snapshot(&mut client, "some-other-host", &rows).await;
    match res {
        ControlResponse::ApplyVmWindowSnapshotAck {
            ok,
            applied,
            detail,
        } => {
            assert!(!ok, "mismatched device_id must reject");
            assert_eq!(applied, 0);
            assert!(detail.contains("device_id"), "detail: {detail}");
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert!(
        rx.recv_timeout(Duration::from_millis(200)).is_err(),
        "rejected snapshot must not push a reload message",
    );

    drop(client);
    server.await.unwrap();
}
