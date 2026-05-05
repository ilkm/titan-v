use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use quinn::{Connection, RecvStream};
use titan_common::{ControlRequest, ControlResponse};

use crate::app::net::{ensure_connection_for_telemetry, exchange_one};

pub(super) async fn start_telemetry_session(quic_addr: &str) -> Result<(Connection, RecvStream)> {
    let connection = ensure_telemetry_connection(quic_addr).await?;
    let response = subscribe_telemetry(quic_addr).await?;
    validate_subscribe_response(response)?;
    let recv = accept_telemetry_uni_stream(&connection).await?;
    Ok((connection, recv))
}

async fn ensure_telemetry_connection(quic_addr: &str) -> Result<Connection> {
    tokio::time::timeout(
        Duration::from_millis(180),
        ensure_connection_for_telemetry(quic_addr),
    )
    .await
    .map_err(|_| anyhow!("ensure connection timeout"))?
}

async fn subscribe_telemetry(quic_addr: &str) -> Result<ControlResponse> {
    tokio::time::timeout(
        Duration::from_millis(220),
        exchange_one(quic_addr, &ControlRequest::SubscribeTelemetry),
    )
    .await
    .map_err(|_| anyhow!("subscribe rpc timeout"))?
}

fn validate_subscribe_response(response: ControlResponse) -> Result<()> {
    match response {
        ControlResponse::SubscribeTelemetryAck { ok: true } => Ok(()),
        ControlResponse::SubscribeTelemetryAck { ok: false } => {
            Err(anyhow!("host refused telemetry subscription"))
        }
        ControlResponse::ServerError { message, .. } => Err(anyhow!("host: {message}")),
        other => Err(anyhow!("unexpected response to subscribe: {other:?}")),
    }
}

async fn accept_telemetry_uni_stream(connection: &Connection) -> Result<RecvStream> {
    tokio::time::timeout(Duration::from_millis(1200), connection.accept_uni())
        .await
        .map_err(|_| anyhow!("accept telemetry uni timeout"))?
        .context("accept telemetry uni stream")
}
