//! User-mode probes for optional **kernel driver IPC** (Phase 2+).

/// Tries a sub-millisecond connect to the host driver named pipe (Windows only).
#[must_use]
pub fn probe_kernel_driver_ipc_blocking() -> bool {
    #[cfg(windows)]
    {
        const PS: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
try {
  $c = New-Object System.IO.Pipes.NamedPipeClientStream('.', 'TitanVHostDriver', [System.IO.Pipes.PipeDirection]::InOut)
  $c.Connect(1)
  $c.Close()
  'true'
} catch {
  'false'
}
"#;
        match std::process::Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(PS)
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim() == "true",
            _ => false,
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}
