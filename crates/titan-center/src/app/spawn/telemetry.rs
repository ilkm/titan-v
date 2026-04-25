//! Duplex telemetry TCP reader (one stream per `host_key`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;

use super::super::constants::TELEMETRY_MAX_CONCURRENT;
use super::super::net_client::{read_telemetry_push, telemetry_addr_for_control};
use super::super::net_msg::NetUiMsg;
use super::super::tcp_tune::tune_connected_stream;
use super::super::{CenterApp, TelemetryLink};

impl CenterApp {
    /// Dedicated telemetry TCP for [`Self::control_addr`] (primary session).
    pub(crate) fn spawn_telemetry_reader(&mut self) {
        let host_key = CenterApp::endpoint_addr_key(&self.control_addr);
        self.spawn_telemetry_reader_for(host_key, self.control_addr.clone());
    }

    /// One telemetry TCP reader per `host_key` (reconnects with backoff until stopped). Fleet cap: [`TELEMETRY_MAX_CONCURRENT`].
    pub(crate) fn spawn_telemetry_reader_for(&mut self, host_key: String, control_addr: String) {
        if host_key.is_empty() || control_addr.trim().is_empty() {
            return;
        }
        if self.telemetry_fleet_cap_blocks(&host_key) {
            return;
        }
        let telemetry_addr = match telemetry_addr_for_control(&control_addr) {
            Ok(a) => a,
            Err(e) => {
                self.send_net_ui_error(format!("telemetry address: {e}"));
                return;
            }
        };
        let Some((gen, stop, running)) = self.telemetry_link_start_or_skip(&host_key) else {
            return;
        };
        let host_key_owned = host_key.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            run_telemetry_thread(host_key_owned, gen, stop, running, telemetry_addr, tx, ctx);
        });
    }

    fn telemetry_fleet_cap_blocks(&mut self, host_key: &str) -> bool {
        let active = self
            .telemetry_links
            .values()
            .filter(|l| l.running.load(Ordering::SeqCst))
            .count();
        let this_running = self
            .telemetry_links
            .get(host_key)
            .is_some_and(|l| l.running.load(Ordering::SeqCst));
        if !this_running && active >= TELEMETRY_MAX_CONCURRENT {
            self.send_net_ui_error(format!(
                "telemetry: max {TELEMETRY_MAX_CONCURRENT} concurrent TCP streams (fleet cap)"
            ));
            return true;
        }
        false
    }

    fn send_net_ui_error(&self, msg: String) {
        let _ = self.net_tx.send(NetUiMsg::Error(msg));
        self.ctx.request_repaint();
    }

    fn telemetry_link_start_or_skip(
        &mut self,
        host_key: &str,
    ) -> Option<(u64, Arc<AtomicBool>, Arc<AtomicBool>)> {
        let link = self
            .telemetry_links
            .entry(host_key.to_string())
            .or_insert_with(|| TelemetryLink {
                session_gen: 0,
                stop: Arc::new(AtomicBool::new(true)),
                running: Arc::new(AtomicBool::new(false)),
            });
        if link.running.load(Ordering::SeqCst) {
            return None;
        }
        link.stop = Arc::new(AtomicBool::new(false));
        link.session_gen = link.session_gen.wrapping_add(1);
        let gen = link.session_gen;
        let stop = link.stop.clone();
        let running = link.running.clone();
        running.store(true, Ordering::SeqCst);
        Some((gen, stop, running))
    }
}

fn telemetry_current_thread_runtime(
    running: &Arc<AtomicBool>,
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
) -> Option<tokio::runtime::Runtime> {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => Some(r),
        Err(e) => {
            running.store(false, Ordering::SeqCst);
            let _ = tx.send(NetUiMsg::Error(format!("telemetry tokio runtime: {e}")));
            ctx.request_repaint();
            None
        }
    }
}

fn run_telemetry_thread(
    host_key: String,
    gen: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    telemetry_addr: String,
    tx: Sender<NetUiMsg>,
    ctx: egui::Context,
) {
    let Some(rt) = telemetry_current_thread_runtime(&running, &tx, &ctx) else {
        return;
    };
    rt.block_on(telemetry_connect_loop(
        host_key,
        gen,
        stop,
        running,
        telemetry_addr,
        tx,
        ctx,
    ));
}

async fn telemetry_run_connected_read(
    mut sock: TcpStream,
    host_key: String,
    gen: u64,
    stop: Arc<AtomicBool>,
    tx: Sender<NetUiMsg>,
    ctx: egui::Context,
) {
    let _ = tune_connected_stream(&sock);
    read_stream_until_disconnect(&mut sock, &host_key, gen, &stop, &tx, &ctx).await;
}

async fn telemetry_backoff_after_connect_err(
    telemetry_addr: &str,
    err: &std::io::Error,
    backoff_ms: u64,
) -> u64 {
    tracing::warn!(
        addr = %telemetry_addr,
        error = %err,
        backoff_ms,
        "telemetry TCP connect failed; retrying"
    );
    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
    (backoff_ms.saturating_mul(2)).min(10_000)
}

async fn telemetry_try_connect_and_read(
    telemetry_addr: &str,
    host_key: &str,
    gen: u64,
    stop: &Arc<AtomicBool>,
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
    backoff_ms: u64,
) -> u64 {
    match TcpStream::connect(telemetry_addr).await {
        Ok(sock) => {
            telemetry_run_connected_read(
                sock,
                host_key.to_string(),
                gen,
                stop.clone(),
                tx.clone(),
                ctx.clone(),
            )
            .await;
            200
        }
        Err(e) => telemetry_backoff_after_connect_err(telemetry_addr, &e, backoff_ms).await,
    }
}

async fn telemetry_connect_loop(
    host_key: String,
    gen: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    telemetry_addr: String,
    tx: Sender<NetUiMsg>,
    ctx: egui::Context,
) {
    let mut backoff_ms: u64 = 200;
    loop {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        backoff_ms = telemetry_try_connect_and_read(
            &telemetry_addr,
            &host_key,
            gen,
            &stop,
            &tx,
            &ctx,
            backoff_ms,
        )
        .await;
    }
    running.store(false, Ordering::SeqCst);
}

fn notify_telemetry_read_failed(
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
    host_key: &str,
    gen: u64,
    err: impl std::fmt::Display,
) {
    tracing::warn!(error = %err, "telemetry TCP read failed; reconnecting");
    let _ = tx.send(NetUiMsg::TelemetryLinkLost {
        host_key: host_key.to_string(),
        gen,
    });
    ctx.request_repaint();
}

async fn read_stream_until_disconnect(
    sock: &mut TcpStream,
    host_key: &str,
    gen: u64,
    stop: &Arc<AtomicBool>,
    tx: &Sender<NetUiMsg>,
    ctx: &egui::Context,
) {
    loop {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        match read_telemetry_push(sock).await {
            Ok(push) => {
                let _ = tx.send(NetUiMsg::HostTelemetry {
                    host_key: host_key.to_string(),
                    gen,
                    push,
                });
                ctx.request_repaint();
            }
            Err(e) => {
                notify_telemetry_read_failed(tx, ctx, host_key, gen, e);
                break;
            }
        }
    }
}
