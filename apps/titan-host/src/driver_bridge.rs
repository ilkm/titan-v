//! User-mode probes for optional **kernel driver IPC** (Phase 2+).

/// Tries to open the host driver named pipe (Windows only); returns whether a handle was obtained.
#[must_use]
pub fn probe_kernel_driver_ipc_blocking() -> bool {
    #[cfg(windows)]
    {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(r"\\.\pipe\TitanVHostDriver")
        {
            Ok(file) => {
                drop(file);
                true
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}
