use std::sync::Arc;
use std::time::Duration;

use quinn::Connection;
use titan_common::ControlPush;
use titan_quic::frame_io;
use tokio::sync::broadcast;

use super::state::ServeState;
use super::telemetry;

const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(50);

pub(super) fn start_background_loops(tx: broadcast::Sender<ControlPush>) {
    spawn_telemetry_resource_live_loop(tx.clone());
    spawn_telemetry_desktop_preview_loop(tx.clone());
    spawn_telemetry_heartbeat_loop(tx);
}

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

fn spawn_telemetry_heartbeat_loop(tx: broadcast::Sender<ControlPush>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            if tx.receiver_count() == 0 {
                continue;
            }
            let _ = tx.send(ControlPush::HostHeartbeat {
                ts_ms: heartbeat_now_ms(),
            });
        }
    });
}

fn heartbeat_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
    if !titan_common::telemetry_push_payload_fits(&push) {
        return;
    }
    let _ = tx.send(push);
}

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

pub(super) fn spawn_telemetry_uni_pump(connection: Connection, state: Arc<ServeState>) {
    tokio::spawn(async move {
        let mut send = match connection.open_uni().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "telemetry: open_uni failed");
                return;
            }
        };
        let mut rx = state.telemetry_tx.subscribe();
        if let Some(initial) = telemetry::build_telemetry_push(None).await
            && let Err(e) = frame_io::write_telemetry_push(&mut send, &initial).await
        {
            tracing::debug!(error = %e, "telemetry: write initial failed");
            return;
        }
        loop {
            match rx.recv().await {
                Ok(push) => {
                    if let Err(e) = frame_io::write_telemetry_push(&mut send, &push).await {
                        tracing::debug!(error = %e, "telemetry: stream closed");
                        return;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    });
}
