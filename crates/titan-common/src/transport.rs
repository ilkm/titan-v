//! Control-plane transports: DDS-style heartbeat + **TCP wire-compatible** gRPC-shaped ping.

use crate::error::{Error, Result};
use crate::{
    decode_control_host_payload, encode_control_request_frame, parse_header, ControlHostFrame,
    ControlRequest, ControlRequestFrame, ControlResponse, FRAME_HEADER_LEN,
};
use std::io::Read;
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

/// Performs one `Ping`/`Pong` exchange over TCP using framed control-plane encoding.
#[derive(Debug, Clone)]
pub struct TcpWirePingClient {
    addr: SocketAddr,
}

impl TcpWirePingClient {
    #[must_use]
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    fn connect_tcp(&self) -> Result<TcpStream> {
        TcpStream::connect(self.addr).map_err(|e| Error::HyperVRejected {
            message: format!("tcp wire ping connect {}: {e}", self.addr),
        })
    }

    fn write_ping_frame(stream: &mut TcpStream) -> Result<()> {
        let frame = encode_control_request_frame(&ControlRequestFrame {
            id: 1,
            body: ControlRequest::Ping,
        })
        .map_err(|e| Error::HyperVRejected {
            message: format!("encode ping: {e}"),
        })?;
        stream.write_all(&frame).map_err(Error::Io)?;
        stream.flush().map_err(Error::Io)?;
        Ok(())
    }

    fn read_host_frame(stream: &mut TcpStream) -> Result<ControlHostFrame> {
        let mut hdr = [0u8; FRAME_HEADER_LEN];
        stream.read_exact(&mut hdr).map_err(Error::Io)?;
        let (_, len) = parse_header(&hdr).map_err(|e| Error::HyperVRejected {
            message: format!("parse header: {e}"),
        })?;
        let mut payload = vec![0u8; len as usize];
        stream.read_exact(&mut payload).map_err(Error::Io)?;
        decode_control_host_payload(&payload).map_err(|e| Error::HyperVRejected {
            message: format!("decode control host frame: {e}"),
        })
    }

    fn expect_pong_response(host: ControlHostFrame) -> Result<()> {
        match host {
            ControlHostFrame::Response {
                id: 1,
                body: ControlResponse::Pong { .. },
            } => Ok(()),
            ControlHostFrame::Response {
                body: ControlResponse::ServerError { code, message },
                ..
            } => Err(Error::HyperVRejected {
                message: format!("host error {code}: {message}"),
            }),
            _ => Err(Error::HyperVRejected {
                message: "unexpected control response to Ping".into(),
            }),
        }
    }
}

impl GrpcControlPlane for TcpWirePingClient {
    fn ping(&self) -> Result<()> {
        let mut stream = self.connect_tcp()?;
        Self::write_ping_frame(&mut stream)?;
        let host = Self::read_host_frame(&mut stream)?;
        Self::expect_pong_response(host)
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
