//! Center-side QUIC client: pool of long-lived connections keyed by `host_quic_addr`.
//!
//! Public surface kept tiny on purpose:
//! * [`init_global`] — bootstrap calls this once with the shared mTLS [`Identity`] + [`TrustStore`].
//! * [`exchange_one`] — fire one RPC on a fresh bi-stream; reuses a cached connection.
//! * [`telemetry_subscribe`] — open a uni-stream reader for one host.
//! * [`forget_host`] — drop a cached connection when the user removes a device.
//!
//! Connection cache: one entry per `host_quic_addr`. Reconnect is lazy (first failed exchange
//! triggers a re-dial); we keep it simple — no exponential backoff inside the cache, callers do
//! their own retry cadence.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, anyhow};
use parking_lot::Mutex;
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use titan_common::{
    ControlHostFrame, ControlPush, ControlRequest, ControlRequestFrame, ControlResponse,
};
use titan_quic::{
    Identity, TrustStore, build_client_config, frame_io, install_default_crypto_provider,
    sni_for_host,
};
use tokio::runtime::Runtime;

static CONTROL_CLIENT: OnceLock<Arc<ControlClient>> = OnceLock::new();
static REQ_ID: AtomicU64 = AtomicU64::new(1);

/// Holds a dedicated multi-thread Tokio runtime alongside the `quinn::Endpoint` driven by it.
///
/// `quinn::Endpoint::client` spawns its UDP driver onto the current Tokio runtime; the Center's
/// egui thread is plain sync at startup, so we own a long-lived multi-thread runtime here whose
/// worker threads keep the endpoint reactor alive. Callers (per-task runtimes elsewhere) then
/// `block_on` `endpoint.connect(...)` futures that talk to the driver via async channels — that
/// works as long as the caller's runtime has `enable_io + enable_time`, which all spawn modules
/// already enable.
pub struct ControlClient {
    endpoint: Endpoint,
    pool: Mutex<HashMap<String, Connection>>,
    /// Owns the Tokio worker threads that drive the QUIC endpoint's UDP reactor.
    /// Dropped only when the process exits (we live in a `OnceLock`).
    _driver_rt: Runtime,
}

impl ControlClient {
    fn new(identity: &Identity, trust: Arc<TrustStore>) -> Result<Self> {
        install_default_crypto_provider();
        let cfg = build_client_config(identity, trust)?;
        let driver_rt = build_driver_runtime()?;
        let endpoint = bind_endpoint_in_runtime(&driver_rt, cfg)?;
        Ok(Self {
            endpoint,
            pool: Mutex::new(HashMap::new()),
            _driver_rt: driver_rt,
        })
    }

    async fn ensure_connection(&self, addr: &str) -> Result<Connection> {
        if let Some(c) = self.pool.lock().get(addr).cloned()
            && c.close_reason().is_none()
        {
            return Ok(c);
        }
        let connection = ensure_connection_dial(&self.endpoint, addr).await?;
        self.pool
            .lock()
            .insert(addr.to_string(), connection.clone());
        Ok(connection)
    }

    fn forget(&self, addr: &str) {
        if let Some(c) = self.pool.lock().remove(addr) {
            c.close(quinn::VarInt::from_u32(0), b"forget");
        }
    }
}

async fn ensure_connection_dial(endpoint: &Endpoint, addr: &str) -> Result<Connection> {
    let socket: SocketAddr = addr
        .parse()
        .with_context(|| format!("invalid host quic addr {addr}"))?;
    let sni = sni_for_host(addr);
    let connecting = endpoint
        .connect(socket, &sni)
        .with_context(|| format!("quinn connect {addr}"))?;
    connecting
        .await
        .map_err(|e| anyhow::Error::from(e).context(format!("quinn handshake {addr}")))
}

pub fn init_global(identity: Arc<Identity>, trust: Arc<TrustStore>) -> Result<Arc<ControlClient>> {
    if let Some(c) = CONTROL_CLIENT.get() {
        return Ok(c.clone());
    }
    let client = Arc::new(ControlClient::new(identity.as_ref(), trust)?);
    let _ = CONTROL_CLIENT.set(client.clone());
    Ok(client)
}

fn build_driver_runtime() -> Result<Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("titan-quic-client")
        .enable_all()
        .build()
        .context("titan-center QUIC driver runtime")
}

fn bind_endpoint_in_runtime(rt: &Runtime, cfg: quinn::ClientConfig) -> Result<Endpoint> {
    let _guard = rt.enter();
    let bind: SocketAddr = "0.0.0.0:0".parse().expect("static");
    let mut endpoint = Endpoint::client(bind).context("quinn client endpoint")?;
    endpoint.set_default_client_config(cfg);
    Ok(endpoint)
}

#[must_use]
pub fn try_get_global() -> Option<Arc<ControlClient>> {
    CONTROL_CLIENT.get().cloned()
}

fn require_global() -> Result<Arc<ControlClient>> {
    try_get_global().ok_or_else(|| anyhow!("control client not initialised"))
}

pub fn forget_host(addr: &str) {
    if let Some(c) = try_get_global() {
        c.forget(addr);
    }
}

pub async fn exchange_one(addr: &str, req: &ControlRequest) -> Result<ControlResponse> {
    let id = REQ_ID.fetch_add(1, Ordering::Relaxed);
    let client = require_global()?;
    let connection = client.ensure_connection(addr).await?;
    let (send, recv) = open_one_bi(&connection).await?;
    perform_one_rpc(send, recv, id, req).await
}

async fn open_one_bi(connection: &Connection) -> Result<(SendStream, RecvStream)> {
    connection
        .open_bi()
        .await
        .map_err(|e| anyhow!("quic open_bi: {e}"))
}

async fn perform_one_rpc(
    mut send: SendStream,
    mut recv: RecvStream,
    id: u64,
    req: &ControlRequest,
) -> Result<ControlResponse> {
    let frame = ControlRequestFrame {
        id,
        body: req.clone(),
    };
    frame_io::write_control_request(&mut send, &frame).await?;
    send.finish().context("quic send.finish")?;
    loop {
        let res = frame_io::read_one_control_host(&mut recv).await?;
        match res {
            Some(ControlHostFrame::Response { id: rid, body }) if rid == id => return Ok(body),
            Some(ControlHostFrame::Response { id: rid, .. }) => {
                return Err(anyhow!("rpc id mismatch (got {rid}, expected {id})"));
            }
            Some(ControlHostFrame::Push(_)) => continue,
            None => return Err(anyhow!("rpc stream closed before response")),
        }
    }
}

/// Ensure a long-lived QUIC connection exists for `addr` and return it for telemetry use.
pub async fn ensure_connection_for_telemetry(addr: &str) -> Result<Connection> {
    let client = require_global()?;
    client.ensure_connection(addr).await
}

/// Read the next telemetry push from the host's telemetry uni stream.
///
/// Caller is responsible for opening the uni stream via `connection.accept_uni().await` once,
/// then passing the `RecvStream` here repeatedly.
pub async fn read_one_telemetry_push(recv: &mut RecvStream) -> Result<Option<ControlPush>> {
    frame_io::read_one_telemetry_push(recv).await
}
