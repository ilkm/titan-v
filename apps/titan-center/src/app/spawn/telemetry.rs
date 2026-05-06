//! Per-host QUIC telemetry reader (one uni-stream per `host_key`).
//!
//! Lifecycle: ensure a long-lived QUIC connection to `host_quic_addr`, send a
//! [`titan_common::ControlRequest::SubscribeTelemetry`] over a fresh bi-stream, then read
//! [`titan_common::ControlPush`] frames from the host-opened uni-stream until the connection
//! drops or the user stops the link. Reconnects with exponential backoff capped at 10s.

mod session;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::time::Duration;

use quinn::{Connection, RecvStream};

use super::super::constants::TELEMETRY_MAX_CONCURRENT;
use super::super::net::{NetUiMsg, forget_host, read_one_telemetry_push};
use super::super::{CenterApp, TelemetryLink};
use session::start_telemetry_session;

impl CenterApp {
    pub(crate) fn spawn_telemetry_reader(&mut self) {
        let host_key = CenterApp::endpoint_addr_key(&self.control_addr);
        self.spawn_telemetry_reader_for(host_key, self.control_addr.clone());
    }

    pub(crate) fn spawn_telemetry_reader_for(&mut self, host_key: String, control_addr: String) {
        if host_key.is_empty() || control_addr.trim().is_empty() {
            return;
        }
        if self.telemetry_fleet_cap_blocks(&host_key) {
            return;
        }
        let Some((session_gen, stop, running)) = self.telemetry_link_start_or_skip(&host_key)
        else {
            return;
        };
        let host_key_owned = host_key.clone();
        let tx = self.net_tx.clone();
        let ctx = self.ctx.clone();
        spawn_telemetry_reader_background(
            host_key_owned,
            session_gen,
            stop,
            running,
            control_addr,
            tx,
            ctx,
        );
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
                "telemetry: max {TELEMETRY_MAX_CONCURRENT} concurrent QUIC streams (fleet cap)"
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
        let session_gen = link.session_gen;
        let stop = link.stop.clone();
        let running = link.running.clone();
        running.store(true, Ordering::SeqCst);
        Some((session_gen, stop, running))
    }
}

fn spawn_telemetry_reader_background(
    host_key: String,
    session_gen: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    quic_addr: String,
    tx: SyncSender<NetUiMsg>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        run_telemetry_thread(host_key, session_gen, stop, running, quic_addr, tx, ctx);
    });
}

fn run_telemetry_thread(
    host_key: String,
    session_gen: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    quic_addr: String,
    tx: SyncSender<NetUiMsg>,
    ctx: egui::Context,
) {
    let Some(rt) = telemetry_current_thread_runtime(&running, &tx, &ctx) else {
        return;
    };
    rt.block_on(telemetry_connect_loop(
        host_key,
        session_gen,
        stop,
        running,
        quic_addr,
        tx,
        ctx,
    ));
}

fn telemetry_current_thread_runtime(
    running: &Arc<AtomicBool>,
    tx: &SyncSender<NetUiMsg>,
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

async fn telemetry_connect_loop(
    host_key: String,
    session_gen: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    quic_addr: String,
    tx: SyncSender<NetUiMsg>,
    ctx: egui::Context,
) {
    let mut backoff_ms: u64 = 50;
    while !stop.load(Ordering::SeqCst) {
        backoff_ms = try_one_session(
            &quic_addr,
            &host_key,
            session_gen,
            &stop,
            &tx,
            &ctx,
            backoff_ms,
        )
        .await;
    }
    running.store(false, Ordering::SeqCst);
}

async fn try_one_session(
    quic_addr: &str,
    host_key: &str,
    session_gen: u64,
    stop: &Arc<AtomicBool>,
    tx: &SyncSender<NetUiMsg>,
    ctx: &egui::Context,
    backoff_ms: u64,
) -> u64 {
    match start_telemetry_session(quic_addr).await {
        Ok((connection, recv)) => {
            read_until_disconnect(connection, recv, host_key, session_gen, stop, tx, ctx).await;
            50
        }
        Err(e) => {
            forget_host(quic_addr);
            tracing::warn!(addr = %quic_addr, error = %e, "telemetry: subscribe failed");
            tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            backoff_ms.saturating_mul(2).min(500)
        }
    }
}

async fn read_until_disconnect(
    _connection: Connection,
    mut recv: RecvStream,
    host_key: &str,
    session_gen: u64,
    stop: &Arc<AtomicBool>,
    tx: &SyncSender<NetUiMsg>,
    ctx: &egui::Context,
) {
    while !stop.load(Ordering::SeqCst) {
        if !read_one_and_forward(&mut recv, host_key, session_gen, tx, ctx).await {
            break;
        }
    }
    let _ = tx.send(NetUiMsg::TelemetryLinkLost {
        host_key: host_key.to_string(),
        session_gen,
    });
    ctx.request_repaint();
}

async fn read_one_and_forward(
    recv: &mut RecvStream,
    host_key: &str,
    session_gen: u64,
    tx: &SyncSender<NetUiMsg>,
    ctx: &egui::Context,
) -> bool {
    match read_one_telemetry_push(recv).await {
        Ok(Some(push)) => {
            maybe_send_decoded_desktop_frame(tx, host_key, &push);
            try_send_drop_telemetry(
                tx,
                NetUiMsg::HostTelemetry {
                    host_key: host_key.to_string(),
                    session_gen,
                    push,
                },
            );
            ctx.request_repaint();
            true
        }
        Ok(None) => false,
        Err(e) => {
            notify_telemetry_read_failed(tx, ctx, host_key, session_gen, e);
            false
        }
    }
}

fn maybe_send_decoded_desktop_frame(
    tx: &SyncSender<NetUiMsg>,
    host_key: &str,
    push: &titan_common::ControlPush,
) {
    let titan_common::ControlPush::HostDesktopPreviewJpeg { jpeg_bytes, .. } = push else {
        return;
    };
    let decoded = match image::load_from_memory(jpeg_bytes) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            tracing::warn!(
                %host_key,
                %e,
                len = jpeg_bytes.len(),
                "telemetry desktop preview: JPEG decode failed in background reader"
            );
            return;
        }
    };
    try_send_drop_telemetry(
        tx,
        NetUiMsg::DesktopFrameDecoded {
            control_addr: host_key.to_string(),
            width: decoded.width() as usize,
            height: decoded.height() as usize,
            rgba_bytes: decoded.into_vec(),
        },
    );
}

fn try_send_drop_telemetry(tx: &SyncSender<NetUiMsg>, msg: NetUiMsg) {
    match tx.try_send(msg) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            tracing::debug!("telemetry: dropped message due to full center inbox");
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

fn notify_telemetry_read_failed(
    tx: &SyncSender<NetUiMsg>,
    ctx: &egui::Context,
    host_key: &str,
    session_gen: u64,
    err: impl std::fmt::Display,
) {
    tracing::warn!(error = %err, "telemetry stream read failed; reconnecting");
    let _ = tx.send(NetUiMsg::TelemetryLinkLost {
        host_key: host_key.to_string(),
        session_gen,
    });
    ctx.request_repaint();
}
