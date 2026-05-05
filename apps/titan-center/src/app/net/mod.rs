//! Control-plane QUIC client, UI↔background thread messages.

mod client;
mod msg;
mod quic_client;

pub use client::{
    capabilities_summary, fetch_desktop_snapshot, fetch_host_resource_snapshot, hello_host,
};
pub use msg::NetUiMsg;
pub use quic_client::{
    ControlClient, ensure_connection_for_telemetry, exchange_one, forget_host, init_global,
    read_one_telemetry_push, try_get_global,
};
