//! GPU-PV / discrete device assignment (DDA) via Hyper-V PowerShell.

use titan_common::{Error, Result};

#[cfg(windows)]
use std::process::{Command, Stdio};

#[cfg(windows)]
fn run_ps(script: &str, env: &[(&str, &str)]) -> Result<()> {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;
    if out.status.success() {
        return Ok(());
    }
    Err(Error::HyperVRejected {
        message: format_ps_fail(out.status.code(), &out.stdout, &out.stderr),
    })
}

#[cfg(windows)]
fn format_ps_fail(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let mut s = format!("exit code: {:?}", code);
    if !stdout.is_empty() {
        s.push_str("\nstdout:\n");
        s.push_str(&stdout);
    }
    if !stderr.is_empty() {
        s.push_str("\nstderr:\n");
        s.push_str(&stderr);
    }
    s
}

#[cfg(windows)]
const PS_ADD_GPU: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$vm = $env:TITAN_VM_NAME
$path = $env:TITAN_GPU_INSTANCE_PATH
if (-not $vm -or -not $path) { throw 'Missing TITAN_VM_NAME or TITAN_GPU_INSTANCE_PATH' }
Add-VMGpuPartitionAdapter -VMName $vm -InstancePath $path
"#;

#[cfg(windows)]
const PS_REMOVE_ALL_GPU: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$vm = $env:TITAN_VM_NAME
if (-not $vm) { throw 'Missing TITAN_VM_NAME' }
Get-VMGpuPartitionAdapter -VMName $vm | Remove-VMGpuPartitionAdapter
"#;

/// Assigns a GPU partition (DDA) to `vm_name` using `partition_instance_id` as `-InstancePath`.
pub fn assign_gpu_partition(vm_name: &str, partition_instance_id: &str) -> Result<()> {
    #[cfg(windows)]
    {
        let vm = vm_name.trim();
        let path = partition_instance_id.trim();
        if vm.is_empty() || path.is_empty() {
            return Err(Error::HyperVRejected {
                message: "vm_name and partition_instance_id must not be empty".into(),
            });
        }
        run_ps(
            PS_ADD_GPU,
            &[("TITAN_VM_NAME", vm), ("TITAN_GPU_INSTANCE_PATH", path)],
        )
    }
    #[cfg(not(windows))]
    {
        let _ = (vm_name, partition_instance_id);
        Err(Error::HyperVRejected {
            message: "GPU-PV assignment requires Windows with Hyper-V.".into(),
        })
    }
}

/// Whether the Hyper-V PowerShell module can be imported (role / RSAT surface).
#[must_use]
pub fn hyperv_ps_module_available_blocking() -> bool {
    #[cfg(windows)]
    {
        const PS: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) { exit 1 }
Import-Module Hyper-V -ErrorAction Stop
"#;
        let out = match Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(PS)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(o) => o,
            Err(_) => return false,
        };
        out.status.success()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
fn powershell_probe_output(cmd: &str) -> Option<std::process::Output> {
    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()
}

#[cfg(windows)]
fn gpu_partition_cmdlets_probe_success() -> bool {
    const PS_PROBE: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) { 'false'; exit 0 }
Import-Module Hyper-V
if (Get-Command Add-VMGpuPartitionAdapter -ErrorAction SilentlyContinue) { 'true' } else { 'false' }
"#;
    let Some(out) = powershell_probe_output(PS_PROBE) else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    String::from_utf8_lossy(&out.stdout).trim() == "true"
}

/// Returns whether `Add-VMGpuPartitionAdapter` is available (Hyper-V PowerShell module present).
///
/// Used for capability bits only; does not prove DDA-capable hardware.
pub fn gpu_partition_cmdlets_available_blocking() -> bool {
    #[cfg(windows)]
    {
        gpu_partition_cmdlets_probe_success()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Target simultaneous GPU-PV slots in the need.md template (honest: hardware and policy still cap reality).
pub const GPU_PV_SLOT_TEMPLATE_MAX: u32 = 40;

/// Guards orchestration when assigning a 0-based slot index in large farms.
pub fn validate_gpu_pv_slot_index(slot: u32) -> Result<()> {
    if slot >= GPU_PV_SLOT_TEMPLATE_MAX {
        return Err(Error::HyperVRejected {
            message: format!("gpu_pv slot {slot} exceeds template max {GPU_PV_SLOT_TEMPLATE_MAX}"),
        });
    }
    Ok(())
}

/// Removes all GPU partition adapters from `vm_name` (best-effort rollback).
pub fn remove_gpu_partition(vm_name: &str) -> Result<()> {
    #[cfg(windows)]
    {
        let vm = vm_name.trim();
        if vm.is_empty() {
            return Err(Error::HyperVRejected {
                message: "vm_name must not be empty".into(),
            });
        }
        run_ps(PS_REMOVE_ALL_GPU, &[("TITAN_VM_NAME", vm)])
    }
    #[cfg(not(windows))]
    {
        let _ = vm_name;
        Err(Error::HyperVRejected {
            message: "GPU-PV removal requires Windows with Hyper-V.".into(),
        })
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    #[test]
    fn slot_within_template_ok() {
        validate_gpu_pv_slot_index(0).unwrap();
        validate_gpu_pv_slot_index(GPU_PV_SLOT_TEMPLATE_MAX - 1).unwrap();
    }

    #[test]
    fn slot_past_template_rejected() {
        assert!(validate_gpu_pv_slot_index(GPU_PV_SLOT_TEMPLATE_MAX).is_err());
    }
}

#[cfg(all(test, windows))]
mod gpu_tests {
    use super::*;

    #[test]
    #[ignore = "requires Hyper-V hardware and DDA-capable GPU"]
    fn assign_gpu_requires_real_hardware() {
        let _ = assign_gpu_partition("nonexistent-vm", "PCI\\VEN_xxxx");
    }
}
