//! VM power state via Hyper-V `Get-VM`.

#[cfg(windows)]
use std::process::{Command, Stdio};

use titan_common::state::VmPowerState;
use titan_common::{Error, Result};

/// Returns VM power state from Hyper-V (`Get-VM`).
pub fn get_vm_power_state_blocking(vm_name: &str) -> Result<VmPowerState> {
    let vm = vm_name.trim();
    if vm.is_empty() {
        return Err(Error::HyperVRejected {
            message: "vm_name must not be empty".into(),
        });
    }
    #[cfg(windows)]
    {
        let esc = vm.replace('\'', "''");
        let script = format!(
            r#"$ErrorActionPreference = 'Stop'
Import-Module Hyper-V
(Get-VM -Name '{esc}' -ErrorAction Stop).State.ToString()"#
        );
        let mut cmd = Command::new("powershell.exe");
        cmd.arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(&script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let out = cmd.output().map_err(|e| Error::HyperVRejected {
            message: format!("powershell: {e}"),
        })?;
        if !out.status.success() {
            return Err(Error::HyperVRejected {
                message: super::ps::format_ps(out.status.code(), &out.stdout, &out.stderr),
            });
        }
        let s = String::from_utf8_lossy(&out.stdout)
            .trim()
            .to_ascii_lowercase();
        Ok(match s.as_str() {
            "off" => VmPowerState::Off,
            "running" => VmPowerState::Running,
            "paused" | "saved" => VmPowerState::Paused,
            _ => VmPowerState::Unknown,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = vm;
        Ok(VmPowerState::Unknown)
    }
}
