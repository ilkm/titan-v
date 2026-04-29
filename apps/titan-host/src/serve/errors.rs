use titan_common::WireError;

/// Errors surfaced by the control server (no guest secrets).
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("wire: {0}")]
    Wire(#[from] WireError),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("connection timed out")]
    Timeout,
    #[error("config: {0}")]
    Config(String),
}
