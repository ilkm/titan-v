//! TCP control plane: framed requests / responses with **multi-frame sessions**.

mod announce;
mod apply_host_ui;
mod dispatch;
mod errors;
mod io;
mod limits;
mod response;
mod run;
mod state;
mod telemetry;
mod vm_window_remote;

pub use announce::HostAnnounceConfig;
pub use errors::ServeError;
pub use run::{ServeUiChannels, handle_connection, run_serve};
pub use state::{ServeState, VmWindowReloadMsg};
