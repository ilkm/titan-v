//! QUIC + mTLS control plane: each Center connection gets its own QUIC connection,
//! every RPC opens its own bi-stream, telemetry rides a single uni-stream per connection.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc as sync_mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use quinn::{Connection, Endpoint, Incoming};
use titan_common::{
    ControlHostFrame, ControlPush, ControlRequest, ControlResponse, HostRuntimeProbes, UiLang,
};
use titan_quic::{
    Identity, Pairing, TrustStore, build_server_config, frame_io, install_default_crypto_provider,
};
use tokio::sync::watch;
use tokio::time::timeout;

use crate::agent_binding_table::AgentBindingTable;
use crate::ui_persist::HostUiPersist;

use super::announce::{HostAnnounceConfig, spawn_host_announce_background};
use super::dispatch::dispatch_request;
use super::errors::ServeError;
use super::limits::DEFAULT_IDLE_BETWEEN_FRAMES;
use super::state::{ServeState, VmWindowReloadMsg};
use super::telemetry;
use super::telemetry_loops;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// `std::sync::mpsc` bridges from the control-plane thread into the Host egui thread.
#[derive(Clone)]
pub struct ServeUiChannels {
    pub persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
    pub lang_apply_tx: Option<sync_mpsc::Sender<UiLang>>,
    pub vm_windows_reload_tx: Option<sync_mpsc::Sender<VmWindowReloadMsg>>,
}

/// Crypto material + trust store + pairing flag injected from the egui side.
pub struct ServeSecurity {
    pub identity: Arc<Identity>,
    pub trust: Arc<TrustStore>,
    pub pairing: Arc<Pairing>,
}

async fn build_serve_state(
    agents: Arc<AgentBindingTable>,
    host_notice: String,
    ui: ServeUiChannels,
) -> Result<Arc<ServeState>, ServeError> {
    let host_notice = std::sync::Mutex::new(host_notice);
    let (gpu_partition_available, runtime_probes) = tokio::task::spawn_blocking(|| {
        (
            false,
            crate::host_runtime_probes::probe_host_runtime_blocking(),
        )
    })
    .await
    .unwrap_or((false, HostRuntimeProbes::default()));
    log_runtime_probes(gpu_partition_available, &runtime_probes);
    Ok(Arc::new(ServeState::new(
        agents,
        host_notice,
        gpu_partition_available,
        runtime_probes,
        ui.persist_apply_tx,
        ui.lang_apply_tx,
        ui.vm_windows_reload_tx,
    )))
}

fn log_runtime_probes(gpu_partition_available: bool, runtime_probes: &HostRuntimeProbes) {
    let caps = &runtime_probes.spoof_host;
    tracing::info!(
        openvmm_wired = runtime_probes.openvmm_wired,
        gpu_partition_supported = gpu_partition_available,
        spoof_network = caps.network_identity,
        spoof_checkpoint = caps.vm_checkpoint_policy,
        spoof_processor = caps.vm_processor_count,
        kernel_driver_ipc = runtime_probes.kernel_driver_ipc,
        "host runtime probes"
    );
}

/// Listens until `shutdown` becomes true or the sender is dropped.
pub async fn run_serve(
    bind: SocketAddr,
    agent_bindings: Arc<AgentBindingTable>,
    agent_bindings_notice: String,
    announce: HostAnnounceConfig,
    security: ServeSecurity,
    shutdown: watch::Receiver<bool>,
    ui_channels: ServeUiChannels,
) -> Result<(), ServeError> {
    install_default_crypto_provider();
    let state = build_serve_state(agent_bindings, agent_bindings_notice, ui_channels).await?;
    let endpoint = build_endpoint(&security, bind).map_err(serve_io_err)?;
    let local = endpoint.local_addr().map_err(ServeError::Io)?;
    spawn_host_announce_background(announce, bind, local, &security.identity);
    tracing::info!(%bind, fingerprint = %security.identity.spki_sha256_hex, "QUIC control + telemetry plane listening");
    telemetry_loops::start_background_loops(state.telemetry_tx.clone());

    accept_loop(endpoint, state, shutdown).await
}

fn build_endpoint(security: &ServeSecurity, bind: SocketAddr) -> Result<Endpoint> {
    let server_cfg = build_server_config(
        security.identity.as_ref(),
        security.trust.clone(),
        Some(security.pairing.clone()),
    )
    .context("build server config")?;
    titan_quic::bind_server_endpoint(bind, server_cfg)
}

fn serve_io_err(err: anyhow::Error) -> ServeError {
    ServeError::Io(std::io::Error::other(err.to_string()))
}

async fn accept_loop(
    endpoint: Endpoint,
    state: Arc<ServeState>,
    shutdown: watch::Receiver<bool>,
) -> Result<(), ServeError> {
    accept_until_shutdown(&endpoint, &state, shutdown).await;
    graceful_shutdown_quic(&endpoint, &state).await;
    Ok(())
}

async fn accept_until_shutdown(
    endpoint: &Endpoint,
    state: &Arc<ServeState>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            res = shutdown.changed() => {
                if res.is_err() {
                    tracing::info!("serve shutdown signal (sender dropped)");
                    return;
                }
                if *shutdown.borrow() {
                    tracing::info!("serve stopped from system tray");
                    return;
                }
            }
            inc = endpoint.accept() => {
                let Some(inc) = inc else {
                    tracing::info!("quic endpoint accept yielded None; closing");
                    return;
                };
                spawn_connection(inc, state.clone());
            }
        }
    }
}

/// Best-effort clean teardown: broadcast `HostByeNow` so existing telemetry uni-streams flush
/// the byte to the wire, give them a short window to do so, then close the endpoint (which
/// emits CONNECTION_CLOSE frames to every active peer). Center observes the byte / close at
/// sub-RTT instead of waiting for the QUIC idle timeout.
async fn graceful_shutdown_quic(endpoint: &Endpoint, state: &Arc<ServeState>) {
    if let Err(e) = state.telemetry_tx.send(ControlPush::HostByeNow) {
        tracing::debug!(error = %e, "telemetry bye broadcast failed");
    }
    tokio::time::sleep(Duration::from_millis(30)).await;
    endpoint.close(quinn::VarInt::from_u32(0), b"bye");
    let _ = tokio::time::timeout(Duration::from_millis(50), endpoint.wait_idle()).await;
}

fn spawn_connection(inc: Incoming, state: Arc<ServeState>) {
    let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
    let span = tracing::info_span!("quic_conn", conn_id);
    tokio::spawn(async move {
        let _enter = span.enter();
        let connection = match inc.await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "quic handshake failed");
                return;
            }
        };
        tracing::info!(remote = %connection.remote_address(), "quic connection accepted");
        if let Err(e) = handle_connection(connection, conn_id, state).await {
            tracing::warn!(error = %e, "quic connection ended with error");
        } else {
            tracing::info!("quic connection closed");
        }
    });
}

/// Runs one QUIC connection's accept_bi() loop until it closes.
pub async fn handle_connection(
    connection: Connection,
    conn_id: u64,
    state: Arc<ServeState>,
) -> Result<(), ServeError> {
    let mut frame_seq: u64 = 0;
    loop {
        match connection.accept_bi().await {
            Ok((mut send, mut recv)) => {
                frame_seq += 1;
                let conn = connection.clone();
                let st = state.clone();
                let seq = frame_seq;
                tokio::spawn(async move {
                    if let Err(e) =
                        serve_one_rpc(conn, &mut send, &mut recv, conn_id, seq, &st).await
                    {
                        tracing::warn!(error = %e, conn_id, seq, "rpc stream ended with error");
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => return Ok(()),
            Err(quinn::ConnectionError::LocallyClosed) => return Ok(()),
            Err(quinn::ConnectionError::ConnectionClosed(_)) => return Ok(()),
            Err(quinn::ConnectionError::TimedOut) => return Ok(()),
            Err(e) => return Err(ServeError::Io(std::io::Error::other(e.to_string()))),
        }
    }
}

async fn serve_one_rpc(
    connection: Connection,
    send: &mut quinn::SendStream,
    recv: &mut quinn::RecvStream,
    conn_id: u64,
    seq: u64,
    state: &Arc<ServeState>,
) -> Result<(), ServeError> {
    let Some(req) = read_one_request_with_idle(recv).await? else {
        return Ok(());
    };
    let request_id = format!("{conn_id}-{seq}");
    tracing::info!(%request_id, body = ?req.body, id = req.id, "control request");
    let want_telemetry = matches!(req.body, ControlRequest::SubscribeTelemetry);
    let res = dispatch_request(req.body.clone(), &request_id, state).await?;
    write_response_and_finish(send, req.id, &res).await?;
    let reuse_vms = if let ControlResponse::VmList { vms } = &res {
        Some(vms.clone())
    } else {
        None
    };
    telemetry::publish_telemetry_after_dispatch(state, reuse_vms, &res);
    if want_telemetry {
        telemetry_loops::spawn_telemetry_uni_pump(connection, state.clone());
    }
    Ok(())
}

async fn read_one_request_with_idle(
    recv: &mut quinn::RecvStream,
) -> Result<Option<titan_common::ControlRequestFrame>, ServeError> {
    timeout(
        DEFAULT_IDLE_BETWEEN_FRAMES,
        frame_io::read_one_control_request(recv),
    )
    .await
    .map_err(|_| ServeError::Timeout)?
    .map_err(map_anyhow)
}

async fn write_response_and_finish(
    send: &mut quinn::SendStream,
    id: u64,
    res: &ControlResponse,
) -> Result<(), ServeError> {
    let response = ControlHostFrame::Response {
        id,
        body: res.clone(),
    };
    frame_io::write_control_host(send, &response)
        .await
        .map_err(map_anyhow)?;
    send.finish()
        .map_err(|e| ServeError::Io(std::io::Error::other(e.to_string())))
}

fn map_anyhow(e: anyhow::Error) -> ServeError {
    ServeError::Io(std::io::Error::other(e.to_string()))
}
