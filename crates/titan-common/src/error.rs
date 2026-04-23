//! Library-facing errors. Avoid logging secrets in `Display`.

use std::io;

use thiserror::Error;

/// Top-level error for `titan-common` consumers.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid provision plan: {0}")]
    InvalidPlan(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Hyper-V operation failed: {message}")]
    HyperVRejected { message: String },

    #[error("timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("{feature} is not implemented yet")]
    NotImplemented { feature: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
