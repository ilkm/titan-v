use std::time::Duration;

pub(crate) const DEFAULT_CONN_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) const DEFAULT_IDLE_BETWEEN_FRAMES: Duration = Duration::from_secs(120);

pub(crate) const MAX_FRAMES_PER_CONNECTION: u32 = 8192;

pub(crate) const MAX_SCRIPT_SOURCE_BYTES: usize = 32 * 1024;
