//! Control-plane TCP client, UI↔background thread messages, and stream tuning.

mod client;
mod msg;
mod tcp_tune;

pub use client::{
    capabilities_summary, exchange_one, fetch_desktop_snapshot, fetch_host_resource_snapshot,
    hello_host, read_telemetry_push, telemetry_addr_for_control,
};
pub use msg::NetUiMsg;
pub use tcp_tune::tune_connected_stream;
