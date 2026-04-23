//! TCP control plane (M2): framed requests / responses with **multi-frame sessions**.

mod dispatch;
mod errors;
mod io;
mod limits;
mod power;
mod response;
mod run;
mod state;

pub use errors::ServeError;
pub use run::{handle_connection, run_serve};
pub use state::ServeState;
