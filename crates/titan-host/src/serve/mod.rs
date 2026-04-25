//! TCP control plane: framed requests / responses with **multi-frame sessions**.

mod announce;
mod apply_host_ui;
mod dispatch;
mod errors;
mod io;
mod limits;
mod power;
mod quic_fleet;
mod response;
mod run;
mod state;
mod telemetry;

pub use announce::HostAnnounceConfig;
pub use errors::ServeError;
pub use run::{handle_connection, run_serve, AgentBindingsSpec};
pub use state::ServeState;
