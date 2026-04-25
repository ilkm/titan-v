use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc as sync_mpsc;
use std::sync::Arc;
use std::time::Duration;

use titan_common::{
    control_plane_quic_addr, control_plane_telemetry_addr, encode_control_host_frame,
    encode_telemetry_push_frame, telemetry_push_payload_fits, ControlHostFrame, ControlPush,
    ControlRequestFrame, ControlResponse, HostRuntimeProbes,
};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::time::timeout;

use dashmap::DashMap;
use tokio::sync::Mutex as AsyncMutex;

use crate::runtime::{self, SCRIPT_QUEUE_CAPACITY};
use crate::ui_persist::HostUiPersist;

use super::announce::{spawn_host_announce_background, HostAnnounceConfig};
use super::dispatch::dispatch_request;
use super::errors::ServeError;
use super::io::read_one_control_request;
use super::limits::{DEFAULT_CONN_TIMEOUT, DEFAULT_IDLE_BETWEEN_FRAMES, MAX_FRAMES_PER_CONNECTION};
use super::quic_fleet::spawn_fleet_quic_listener;
use super::state::ServeState;
use super::telemetry;

use crate::tcp_tune::tcp_listen_tokio;

use titan_vmm::hyperv::AgentBindingTable;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// How to obtain the VM→agent address table for [`ServeState`].
#[derive(Clone, Debug)]
pub enum AgentBindingsSpec {
    /// Load from disk (or empty if `None`).
    Path(Option<PathBuf>),
    /// Already materialized (e.g. from GUI persistence).
    Inline {
        agents: Arc<AgentBindingTable>,
        notice: String,
    },
}

async fn build_serve_state_inner(
    agents: Arc<AgentBindingTable>,
    host_notice: String,
    persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
) -> Result<Arc<ServeState>, ServeError> {
    let host_notice = std::sync::Mutex::new(host_notice);
    let (gpu_partition_available, runtime_probes) = tokio::task::spawn_blocking(|| {
        (
            titan_vmm::hyperv::gpu_pv::gpu_partition_cmdlets_available_blocking(),
            crate::host_runtime_probes::probe_host_runtime_blocking(),
        )
    })
    .await
    .unwrap_or((false, HostRuntimeProbes::default()));
    log_runtime_probes(gpu_partition_available, &runtime_probes);
    let (script_tx, script_rx) = mpsc::channel(SCRIPT_QUEUE_CAPACITY);
    let vm_locks = Arc::new(DashMap::<String, Arc<AsyncMutex<()>>>::new());
    tokio::spawn(runtime::script_worker(script_rx, vm_locks));
    runtime::spawn_coordinator_ticks();
    Ok(Arc::new(ServeState::new(
        agents,
        host_notice,
        script_tx,
        gpu_partition_available,
        runtime_probes,
        persist_apply_tx,
    )))
}

async fn build_serve_state_from_spec(
    spec: &AgentBindingsSpec,
    persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
) -> Result<Arc<ServeState>, ServeError> {
    match spec {
        AgentBindingsSpec::Path(path) => {
            let (table, bindings_notice) = crate::agent_bindings::load_or_empty(path.as_deref());
            build_serve_state_inner(
                Arc::new(table),
                bindings_notice.unwrap_or_default(),
                persist_apply_tx,
            )
            .await
        }
        AgentBindingsSpec::Inline { agents, notice } => {
            build_serve_state_inner(agents.clone(), notice.clone(), persist_apply_tx).await
        }
    }
}

fn log_runtime_probes(gpu_partition_available: bool, runtime_probes: &HostRuntimeProbes) {
    let caps = &runtime_probes.spoof_host;
    tracing::info!(
        hyperv_ps_module = runtime_probes.hyperv_ps_module_available,
        linux_virsh = runtime_probes.linux_virsh_available,
        gpu_partition_cmdlets = gpu_partition_available,
        spoof_network = caps.network_identity,
        spoof_checkpoint = caps.vm_checkpoint_policy,
        spoof_processor = caps.vm_processor_count,
        kernel_driver_ipc = runtime_probes.kernel_driver_ipc,
        "host runtime probes"
    );
}

async fn write_telemetry_frame(sock: &mut TcpStream, push: &ControlPush) -> std::io::Result<()> {
    let frame =
        encode_telemetry_push_frame(push).map_err(|e| std::io::Error::other(e.to_string()))?;
    sock.write_all(&frame).await?;
    // No per-frame flush: coalesce with TCP stack for high-rate desktop preview (live feel).
    Ok(())
}

/// Push-only telemetry TCP: center connects and receives `ControlPush` frames (no requests).
/// Pushes [`ControlPush::HostResourceLive`] on a fixed cadence while any telemetry TCP client is subscribed.
fn spawn_telemetry_resource_live_loop(tx: broadcast::Sender<ControlPush>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            if tx.receiver_count() == 0 {
                continue;
            }
            let Ok(stats) =
                tokio::task::spawn_blocking(crate::host_resources::collect_blocking).await
            else {
                continue;
            };
            let _ = tx.send(ControlPush::HostResourceLive { stats });
        }
    });
}

async fn telemetry_desktop_preview_tick(tx: &broadcast::Sender<ControlPush>) {
    const MAX_W: u32 = 640;
    const MAX_H: u32 = 360;
    const JPEG_Q: u8 = 38;
    if tx.receiver_count() == 0 {
        return;
    }
    let cap_res = tokio::task::spawn_blocking(move || {
        crate::desktop_snapshot::capture_primary_display_jpeg(MAX_W, MAX_H, JPEG_Q)
    })
    .await;
    let Ok(Ok((jpeg_bytes, width_px, height_px))) = cap_res else {
        return;
    };
    let push = ControlPush::HostDesktopPreviewJpeg {
        jpeg_bytes,
        width_px,
        height_px,
    };
    if !telemetry_push_payload_fits(&push) {
        return;
    }
    let _ = tx.send(push);
}

/// Desktop JPEG over telemetry at ~3 FPS (device-card thumbnail scenario).
fn spawn_telemetry_desktop_preview_loop(tx: broadcast::Sender<ControlPush>) {
    const TICK: Duration = Duration::from_millis(333);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TICK);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            telemetry_desktop_preview_tick(&tx).await;
        }
    });
}

async fn telemetry_accept_one_client(
    mut sock: TcpStream,
    peer: SocketAddr,
    state: Arc<ServeState>,
) {
    let _ = sock.set_nodelay(true);
    let mut rx = state.telemetry_tx.subscribe();
    tracing::info!(%peer, "telemetry subscriber connected");
    if let Some(initial) = telemetry::build_telemetry_push(None).await {
        let _ = write_telemetry_frame(&mut sock, &initial).await;
        let _ = sock.flush().await;
    }
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(push) => {
                    if write_telemetry_frame(&mut sock, &push).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn spawn_telemetry_accept_loop(listener: TcpListener, state: Arc<ServeState>) {
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((sock, peer)) => {
                    let st = state.clone();
                    tokio::spawn(telemetry_accept_one_client(sock, peer, st));
                }
                Err(e) => tracing::warn!(error = %e, "telemetry accept"),
            }
        }
    });
}

/// Listens until `shutdown` becomes true or the sender is dropped.
///
/// System tray / window lifecycle should own the matching [`watch::Sender`] and call
/// [`spawn_tray_shutdown_for_serve`](titan_tray::spawn_tray_shutdown_for_serve) or equivalent.
pub async fn run_serve(
    bind: SocketAddr,
    agent_bindings: AgentBindingsSpec,
    announce: HostAnnounceConfig,
    shutdown: watch::Receiver<bool>,
    persist_apply_tx: Option<sync_mpsc::Sender<HostUiPersist>>,
) -> Result<(), ServeError> {
    let state = build_serve_state_from_spec(&agent_bindings, persist_apply_tx).await?;
    let listener = tcp_listen_tokio(bind).map_err(ServeError::Io)?;
    let local = listener.local_addr().map_err(ServeError::Io)?;
    spawn_host_announce_background(announce, bind, local);

    let telemetry_bind = control_plane_telemetry_addr(bind);
    let telemetry_listener = tcp_listen_tokio(telemetry_bind).map_err(ServeError::Io)?;
    tracing::info!(%bind, "control-plane command TCP listening");
    tracing::info!(%telemetry_bind, "control-plane telemetry TCP (push-only) listening");
    spawn_telemetry_accept_loop(telemetry_listener, state.clone());
    spawn_telemetry_resource_live_loop(state.telemetry_tx.clone());
    spawn_telemetry_desktop_preview_loop(state.telemetry_tx.clone());

    let quic_bind = control_plane_quic_addr(bind);
    spawn_fleet_quic_listener(quic_bind);
    tracing::info!(%quic_bind, "fleet QUIC UDP (experimental) alongside TCP control plane");

    accept_loop(listener, state, shutdown).await
}

fn spawn_control_connection(sock: TcpStream, peer: SocketAddr, state: Arc<ServeState>) {
    tokio::spawn(async move {
        let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
        let span = tracing::info_span!("control_conn", conn_id, %peer);
        let _enter = span.enter();
        if let Err(e) = handle_connection(sock, DEFAULT_CONN_TIMEOUT, conn_id, state).await {
            tracing::warn!(error = %e, "control connection closed with error");
        } else {
            tracing::info!("control connection closed");
        }
    });
}

async fn accept_loop(
    listener: TcpListener,
    state: Arc<ServeState>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), ServeError> {
    loop {
        tokio::select! {
            res = shutdown.changed() => {
                if res.is_err() {
                    tracing::info!("serve shutdown signal (sender dropped)");
                    return Ok(());
                }
                if *shutdown.borrow() {
                    tracing::info!("serve stopped from system tray");
                    return Ok(());
                }
            }
            accept_res = listener.accept() => {
                let (sock, peer) = accept_res.map_err(ServeError::Io)?;
                spawn_control_connection(sock, peer, state.clone());
            }
        }
    }
}

/// Handles one client session: read frames in a loop until idle timeout, EOF, or cap.
pub async fn handle_connection(
    mut sock: TcpStream,
    session_deadline: Duration,
    conn_id: u64,
    state: Arc<ServeState>,
) -> Result<(), ServeError> {
    timeout(session_deadline, session_loop(&mut sock, conn_id, &state))
        .await
        .map_err(|_| ServeError::Timeout)?
}

async fn session_dispatch_one(
    sock: &mut TcpStream,
    req: ControlRequestFrame,
    state: &Arc<ServeState>,
    conn_id: u64,
    frame_seq: u64,
) -> Result<(), ServeError> {
    let request_id = format!("{conn_id}-{frame_seq}");
    tracing::info!(%request_id, body = ?req.body, id = req.id, "control request");
    let res = dispatch_request(req.body.clone(), &request_id, state).await?;
    let reuse_vms = match &res {
        ControlResponse::VmList { vms } => Some(vms.clone()),
        _ => None,
    };
    let frame = encode_control_host_frame(&ControlHostFrame::Response {
        id: req.id,
        body: res.clone(),
    })?;
    sock.write_all(&frame).await?;
    sock.flush().await?;
    telemetry::publish_telemetry_after_dispatch(state, reuse_vms, &res);
    Ok(())
}

async fn session_loop(
    sock: &mut TcpStream,
    conn_id: u64,
    state: &Arc<ServeState>,
) -> Result<(), ServeError> {
    let mut frame_seq: u64 = 0;
    loop {
        if frame_seq >= u64::from(MAX_FRAMES_PER_CONNECTION) {
            tracing::warn!(conn_id, "max frames per connection reached; closing");
            break;
        }
        let req = match timeout(DEFAULT_IDLE_BETWEEN_FRAMES, read_one_control_request(sock)).await {
            Ok(Ok(Some(r))) => r,
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(ServeError::Timeout),
        };
        frame_seq += 1;
        session_dispatch_one(sock, req, state, conn_id, frame_seq).await?;
    }
    Ok(())
}
