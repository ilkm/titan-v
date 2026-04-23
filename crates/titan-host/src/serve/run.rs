use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use titan_common::{encode_response_frame, HostRuntimeProbes};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;

use dashmap::DashMap;
use tokio::sync::Mutex as AsyncMutex;

use crate::runtime::{self, SCRIPT_QUEUE_CAPACITY};

use super::dispatch::dispatch_request;
use super::errors::ServeError;
use super::io::read_one_request;
use super::limits::{DEFAULT_CONN_TIMEOUT, DEFAULT_IDLE_BETWEEN_FRAMES, MAX_FRAMES_PER_CONNECTION};
use super::state::ServeState;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

async fn build_serve_state(agent_bindings: Option<PathBuf>) -> Result<Arc<ServeState>, ServeError> {
    let table = crate::agent_bindings::load_or_empty(agent_bindings.as_deref())
        .map_err(|e| ServeError::Config(e.to_string()))?;
    let agents = Arc::new(table);
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
        agent_bindings.clone(),
        script_tx,
        gpu_partition_available,
        runtime_probes,
    )))
}

fn log_runtime_probes(gpu_partition_available: bool, runtime_probes: &HostRuntimeProbes) {
    let caps = &runtime_probes.spoof_host;
    tracing::info!(
        gpu_partition_cmdlets = gpu_partition_available,
        spoof_network = caps.network_identity,
        spoof_checkpoint = caps.vm_checkpoint_policy,
        spoof_processor = caps.vm_processor_count,
        kernel_driver_ipc = runtime_probes.kernel_driver_ipc,
        "host runtime probes"
    );
}

/// Listens until the process is interrupted; each accepted socket shares `state`.
pub async fn run_serve(
    bind: SocketAddr,
    agent_bindings: Option<PathBuf>,
) -> Result<(), ServeError> {
    let state = build_serve_state(agent_bindings).await?;
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "control plane listening");
    accept_loop(listener, state).await
}

async fn accept_loop(listener: TcpListener, state: Arc<ServeState>) -> Result<(), ServeError> {
    loop {
        let (sock, peer) = listener.accept().await?;
        let st = state.clone();
        tokio::spawn(async move {
            let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
            let span = tracing::info_span!("control_conn", conn_id, %peer);
            let _enter = span.enter();
            if let Err(e) = handle_connection(sock, DEFAULT_CONN_TIMEOUT, conn_id, st).await {
                tracing::warn!(error = %e, "control connection closed with error");
            } else {
                tracing::info!("control connection closed");
            }
        });
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
        let req = match timeout(DEFAULT_IDLE_BETWEEN_FRAMES, read_one_request(sock)).await {
            Ok(Ok(Some(r))) => r,
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(ServeError::Timeout),
        };
        frame_seq += 1;
        let request_id = format!("{conn_id}-{frame_seq}");
        tracing::info!(%request_id, ?req, "control request");
        let res = dispatch_request(req, &request_id, state).await?;
        let frame = encode_response_frame(&res)?;
        sock.write_all(&frame).await?;
        sock.flush().await?;
    }
    Ok(())
}
