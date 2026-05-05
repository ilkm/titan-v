//! Integration: Center-side `vm_window_db` CRUD round-trip + `vm_window_push_to_hosts`
//! `ApplyVmWindowSnapshot` wire round-trip against a mock host TCP listener.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tempfile::TempDir;
use titan_center::app::vm_window_db;
use titan_center::app::vm_window_push_to_hosts;
use titan_common::{
    ControlHostFrame, ControlRequest, ControlResponse, VmWindowRecord, encode_control_host_frame,
    parse_header, read_control_request_frame,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

fn sample_row(record_id: &str, vm_id: u32, device_id: &str) -> VmWindowRecord {
    VmWindowRecord {
        record_id: record_id.to_string(),
        device_id: device_id.to_string(),
        host_control_addr: "127.0.0.1:7788".to_string(),
        host_label: "lab-host".to_string(),
        cpu_count: 4,
        memory_mib: 8192,
        disk_mib: 65_536,
        vm_directory: format!("vm-{vm_id}"),
        vm_id,
        created_at_unix_ms: 1_700_000_000_000,
    }
}

fn fresh_db() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("vm_windows.sqlite");
    (dir, path)
}

#[test]
fn db_upsert_list_conflict_delete_round_trip() {
    let (_keep, path) = fresh_db();
    assert!(vm_window_db::list_all(&path).unwrap().is_empty());

    let row = sample_row("rec-1", 100, "device-A");
    vm_window_db::upsert(&path, &row).unwrap();

    let after = vm_window_db::list_all(&path).unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].record_id, "rec-1");
    assert_eq!(after[0].vm_id, 100);

    assert!(
        vm_window_db::conflicts_for(&path, "rec-2", 100, "vm-different").unwrap(),
        "different record_id with same vm_id must conflict"
    );
    assert!(
        vm_window_db::conflicts_for(&path, "rec-2", 999, "vm-100").unwrap(),
        "different record_id with same vm_directory must conflict"
    );
    assert!(
        !vm_window_db::conflicts_for(&path, "rec-1", 100, "vm-100").unwrap(),
        "same record_id is updating itself; no conflict",
    );

    let removed = vm_window_db::delete_by_record_id(&path, "rec-1").unwrap();
    assert_eq!(removed, 1);
    assert!(vm_window_db::list_all(&path).unwrap().is_empty());
}

async fn read_request_frame(sock: &mut tokio::net::TcpStream) -> titan_common::ControlRequestFrame {
    let mut hdr = [0u8; titan_common::FRAME_HEADER_LEN];
    sock.read_exact(&mut hdr).await.unwrap();
    let (_, len) = parse_header(&hdr).unwrap();
    let mut payload = vec![0u8; len as usize];
    sock.read_exact(&mut payload).await.unwrap();
    let mut buf = Vec::new();
    buf.extend_from_slice(&hdr);
    buf.extend_from_slice(&payload);
    read_control_request_frame(&mut buf.as_slice()).unwrap()
}

fn unpack_apply_snapshot(req: ControlRequest) -> (String, Vec<VmWindowRecord>) {
    match req {
        ControlRequest::ApplyVmWindowSnapshot {
            device_id,
            records_json,
        } => {
            let parsed: Vec<VmWindowRecord> = serde_json::from_str(&records_json).unwrap();
            (device_id, parsed)
        }
        other => panic!("expected ApplyVmWindowSnapshot, got {other:?}"),
    }
}

async fn write_ack(sock: &mut tokio::net::TcpStream, id: u64, applied: u32) {
    let ack = ControlHostFrame::Response {
        id,
        body: ControlResponse::ApplyVmWindowSnapshotAck {
            ok: true,
            applied,
            detail: String::new(),
        },
    };
    let frame = encode_control_host_frame(&ack).unwrap();
    sock.write_all(&frame).await.unwrap();
    sock.flush().await.unwrap();
}

async fn handle_one_apply_snapshot(
    listener: TcpListener,
    seen: Arc<AtomicUsize>,
    expected_device: String,
    expected_records: Vec<VmWindowRecord>,
) {
    let (mut sock, _) = listener.accept().await.unwrap();
    let req_frame = read_request_frame(&mut sock).await;
    let id = req_frame.id;
    let (device_id, parsed) = unpack_apply_snapshot(req_frame.body);
    assert_eq!(device_id, expected_device);
    assert_eq!(parsed.len(), expected_records.len());
    seen.fetch_add(1, Ordering::SeqCst);
    write_ack(&mut sock, id, parsed.len() as u32).await;
}

fn spawn_mock_host(
    expected_device: String,
    expected_records: Vec<VmWindowRecord>,
) -> (std::net::SocketAddr, Arc<AtomicUsize>, JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    let seen = Arc::new(AtomicUsize::new(0));
    let seen_for_task = seen.clone();
    let join = tokio::spawn(handle_one_apply_snapshot(
        listener,
        seen_for_task,
        expected_device,
        expected_records,
    ));
    (addr, seen, join)
}

#[tokio::test]
async fn tcp_apply_snapshot_round_trip_with_mock_host() {
    let device_id = "device-A".to_string();
    let row = sample_row("rec-1", 100, &device_id);
    let rows = vec![row];

    let (addr, seen, join) = spawn_mock_host(device_id.clone(), rows.clone());

    let req = ControlRequest::ApplyVmWindowSnapshot {
        device_id: device_id.clone(),
        records_json: serde_json::to_string(&rows).unwrap(),
    };
    let addr_str = addr.to_string();
    let blocking = tokio::task::spawn_blocking(move || {
        vm_window_push_to_hosts::tcp_apply_snapshot(&addr_str, &req)
    });
    blocking.await.unwrap().unwrap();

    tokio::time::timeout(Duration::from_secs(2), join)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(seen.load(Ordering::SeqCst), 1);
}
