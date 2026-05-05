//! QUIC + mTLS control plane: framed RPCs on bi-streams + telemetry on a uni-stream.

mod announce;
mod apply_host_ui;
mod dispatch;
mod errors;
mod limits;
mod response;
mod run;
mod state;
mod telemetry;
mod vm_window_remote;

pub use announce::HostAnnounceConfig;
pub use errors::ServeError;
pub use run::{ServeSecurity, ServeUiChannels, handle_connection, run_serve};
pub use state::{ServeState, VmWindowReloadMsg};
