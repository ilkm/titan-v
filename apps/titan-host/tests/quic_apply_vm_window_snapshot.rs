//! End-to-end smoke test: a QUIC client (mTLS) sends `ApplyVmWindowSnapshot` to the host's
//! QUIC server and sees the egui reload mailbox receive a `VmWindowReloadMsg::Replace`.

use std::sync::Arc;
use std::time::Duration;

use tempfile::tempdir;
use titan_common::{ControlRequest, ControlResponse, VmWindowRecord};
use titan_host::serve::ServeState;
use titan_quic::{
    Pairing, Role, TrustEntry, TrustStore, build_client_config, build_server_config,
    install_default_crypto_provider, load_or_generate, sni_for_host,
};

const ALPN_FRAME_TIMEOUT_MS: u64 = 2_000;

#[test]
fn apply_snapshot_round_trip() {
    install_default_crypto_provider();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        run_apply_snapshot_round_trip().await;
    });
}

struct EndPair {
    host_id: &'static str,
    host_identity: Arc<titan_quic::Identity>,
    center_identity: Arc<titan_quic::Identity>,
    host_trust: Arc<TrustStore>,
    center_trust: Arc<TrustStore>,
    pairing: Arc<Pairing>,
}

fn build_test_pair(dir_path: &std::path::Path) -> EndPair {
    let host_id = "test-host-id";
    let center_id = "test-center-id";
    let identities = build_test_identities(dir_path, host_id, center_id);
    let stores = build_test_stores(dir_path);
    seed_test_trust(&identities, &stores, host_id, center_id);
    let pairing = Pairing::new(stores.host_trust.clone());
    EndPair {
        host_id,
        host_identity: identities.host,
        center_identity: identities.center,
        host_trust: stores.host_trust,
        center_trust: stores.center_trust,
        pairing,
    }
}

struct TestIdentities {
    host: Arc<titan_quic::Identity>,
    center: Arc<titan_quic::Identity>,
}

struct TestStores {
    host_trust: Arc<TrustStore>,
    center_trust: Arc<TrustStore>,
}

fn build_test_identities(dir: &std::path::Path, host_id: &str, center_id: &str) -> TestIdentities {
    TestIdentities {
        host: Arc::new(load_or_generate(&dir.join("host"), Role::Host, host_id).unwrap()),
        center: Arc::new(load_or_generate(&dir.join("center"), Role::Center, center_id).unwrap()),
    }
}

fn build_test_stores(dir: &std::path::Path) -> TestStores {
    TestStores {
        host_trust: Arc::new(TrustStore::open(dir.join("host_trust.json")).unwrap()),
        center_trust: Arc::new(TrustStore::open(dir.join("center_trust.json")).unwrap()),
    }
}

fn seed_test_trust(ids: &TestIdentities, stores: &TestStores, host_id: &str, center_id: &str) {
    seed_trust(
        &stores.host_trust,
        &ids.center.spki_sha256_hex,
        "center",
        center_id,
    );
    seed_trust(
        &stores.center_trust,
        &ids.host.spki_sha256_hex,
        "host",
        host_id,
    );
}

async fn run_apply_snapshot_round_trip() {
    let dir = tempdir().unwrap();
    let pair = build_test_pair(dir.path());
    let (server_addr, server_handle, reload_rx) = start_test_server(&pair);
    let connection = connect_test_client(&pair, server_addr).await;
    let row = sample_vm_window(server_addr, pair.host_id);
    // Server validates `device_id` against `host_device_id_string()` (machine-derived); leave
    // empty so the host accepts the snapshot in any test environment.
    let req = ControlRequest::ApplyVmWindowSnapshot {
        device_id: String::new(),
        records_json: serde_json::to_string(&vec![row]).unwrap(),
    };
    let res = run_one_rpc(&connection, &req).await;
    assert_apply_ack(&res);
    assert_reload_received(&reload_rx);
    connection.close(0u32.into(), b"done");
    let _ = server_handle.await;
}

fn start_test_server(
    pair: &EndPair,
) -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<()>,
    std::sync::mpsc::Receiver<titan_host::serve::VmWindowReloadMsg>,
) {
    let server_cfg = build_server_config(
        pair.host_identity.as_ref(),
        pair.host_trust.clone(),
        Some(pair.pairing.clone()),
    )
    .unwrap();
    let bind = "127.0.0.1:0".parse().unwrap();
    let endpoint = titan_quic::bind_server_endpoint(bind, server_cfg).unwrap();
    let server_addr = endpoint.local_addr().unwrap();
    let (reload_tx, reload_rx) = std::sync::mpsc::channel();
    let state = ServeState::for_test_with_reload_tx(reload_tx);
    let server_state = state.clone();
    let handle = tokio::spawn(async move {
        let conn = endpoint.accept().await.unwrap().await.unwrap();
        titan_host::serve::handle_connection(conn, 1, server_state)
            .await
            .unwrap();
    });
    (server_addr, handle, reload_rx)
}

async fn connect_test_client(
    pair: &EndPair,
    server_addr: std::net::SocketAddr,
) -> quinn::Connection {
    let client_cfg =
        build_client_config(pair.center_identity.as_ref(), pair.center_trust.clone()).unwrap();
    let mut client_endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    client_endpoint.set_default_client_config(client_cfg);
    client_endpoint
        .connect(server_addr, &sni_for_host(pair.host_id))
        .unwrap()
        .await
        .unwrap()
}

fn sample_vm_window(server_addr: std::net::SocketAddr, host_id: &str) -> VmWindowRecord {
    VmWindowRecord {
        record_id: "rec1".into(),
        device_id: host_id.into(),
        host_control_addr: server_addr.to_string(),
        host_label: "h".into(),
        cpu_count: 2,
        memory_mib: 1024,
        disk_mib: 4096,
        vm_directory: "/tmp/vm/100".into(),
        vm_id: 100,
        remark: String::new(),
        created_at_unix_ms: 1,
    }
}

fn assert_apply_ack(res: &ControlResponse) {
    match res {
        ControlResponse::ApplyVmWindowSnapshotAck {
            ok: true,
            applied: 1,
            ..
        } => {}
        other => panic!("unexpected ack: {other:?}"),
    }
}

fn assert_reload_received(
    reload_rx: &std::sync::mpsc::Receiver<titan_host::serve::VmWindowReloadMsg>,
) {
    let msg = reload_rx
        .recv_timeout(Duration::from_millis(ALPN_FRAME_TIMEOUT_MS))
        .expect("reload msg never arrived");
    match msg {
        titan_host::serve::VmWindowReloadMsg::Replace { records } => {
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].record_id, "rec1");
        }
    }
}

fn seed_trust(store: &Arc<TrustStore>, fingerprint: &str, role: &str, label: &str) {
    store
        .upsert(TrustEntry {
            fingerprint: fingerprint.to_string(),
            label: label.to_string(),
            role: role.to_string(),
            source: "test".to_string(),
            added_at_epoch_s: 1,
        })
        .unwrap();
}

async fn run_one_rpc(connection: &quinn::Connection, req: &ControlRequest) -> ControlResponse {
    use titan_common::{ControlHostFrame, ControlRequestFrame};
    use titan_quic::frame_io;
    let (mut send, mut recv) = connection.open_bi().await.unwrap();
    let frame = ControlRequestFrame {
        id: 99,
        body: req.clone(),
    };
    frame_io::write_control_request(&mut send, &frame)
        .await
        .unwrap();
    send.finish().unwrap();
    loop {
        match frame_io::read_one_control_host(&mut recv).await.unwrap() {
            Some(ControlHostFrame::Response { id: 99, body }) => return body,
            Some(ControlHostFrame::Push(_)) => continue,
            Some(other) => panic!("unexpected frame: {other:?}"),
            None => panic!("stream closed before response"),
        }
    }
}
