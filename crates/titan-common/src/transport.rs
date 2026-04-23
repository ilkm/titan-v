//! Control-plane transports: DDS-style heartbeat + **TCP wire-compatible** gRPC-shaped ping.

use crate::error::{Error, Result};
use crate::{encode_request_frame, read_response_frame, ControlRequest, ControlResponse};
use std::io::Write;
use std::net::{SocketAddr, TcpStream};

/// Future: DDS topics for fan-out telemetry and low-latency host events.
pub trait DdsControlBus: Send + Sync {
    /// Publishes node capabilities / heartbeat.
    fn publish_heartbeat(&self) -> Result<()>;
}

/// Future: gRPC control channel; [`TcpWirePingClient`] uses the same framed protocol as `serve`.
pub trait GrpcControlPlane: Send + Sync {
    fn ping(&self) -> Result<()>;
}

/// No-op DDS bus (returns [`Error::NotImplemented`]).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopDdsControlBus;

/// No-op gRPC client (returns [`Error::NotImplemented`]).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopGrpcControlPlane;

impl DdsControlBus for NoopDdsControlBus {
    fn publish_heartbeat(&self) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "DDS control bus",
        })
    }
}

impl GrpcControlPlane for NoopGrpcControlPlane {
    fn ping(&self) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "gRPC control plane",
        })
    }
}

/// Writes a structured heartbeat to logs (concrete stand-in for DDS).
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingHeartbeatBus;

impl DdsControlBus for TracingHeartbeatBus {
    fn publish_heartbeat(&self) -> Result<()> {
        tracing::info!(
            target: "titan_common::transport",
            "dds-style heartbeat (tracing only)"
        );
        Ok(())
    }
}

/// Performs one `Ping`/`Pong` exchange over TCP using M2 wire framing.
#[derive(Debug, Clone)]
pub struct TcpWirePingClient {
    addr: SocketAddr,
}

impl TcpWirePingClient {
    #[must_use]
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

impl GrpcControlPlane for TcpWirePingClient {
    fn ping(&self) -> Result<()> {
        let mut stream = TcpStream::connect(self.addr).map_err(|e| Error::HyperVRejected {
            message: format!("tcp wire ping connect {}: {e}", self.addr),
        })?;
        let frame =
            encode_request_frame(&ControlRequest::Ping).map_err(|e| Error::HyperVRejected {
                message: format!("encode ping: {e}"),
            })?;
        stream.write_all(&frame).map_err(Error::Io)?;
        stream.flush().map_err(Error::Io)?;
        let res = read_response_frame(&mut stream).map_err(|e| Error::HyperVRejected {
            message: format!("read pong: {e}"),
        })?;
        match res {
            ControlResponse::Pong { .. } => Ok(()),
            ControlResponse::ServerError { code, message } => Err(Error::HyperVRejected {
                message: format!("host error {code}: {message}"),
            }),
            _ => Err(Error::HyperVRejected {
                message: "unexpected control response to Ping".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_heartbeat_ok() {
        TracingHeartbeatBus.publish_heartbeat().unwrap();
    }

    #[test]
    fn noop_grpc_is_not_implemented() {
        let err = NoopGrpcControlPlane.ping().unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }
}
