//! Windows-only PowerShell invocation. Paths are passed via env vars to avoid quoting bugs.
//!
//! **Requires**: elevated PowerShell / admin rights for `New-VM`, Hyper-V role installed,
//! valid parent VHDX, and optionally an existing virtual switch name.
//!
//! **Timeout**: wall-clock limits are enforced by the caller (`tokio::time::timeout` around
//! `spawn_blocking`). `Command::output` itself is not cancellable; a hung `powershell.exe` may
//! outlive the outer timeout.

use std::process::{Command, Stdio};

use titan_common::{state::VmPowerState, Error, Result, VmProvisionPlan};

use super::gpu_pv::hyperv_ps_module_available_blocking;

const PS_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$parent = $env:TITAN_PARENT_VHDX
$diff = $env:TITAN_DIFF_VHDX
$name = $env:TITAN_VM_NAME
$mem = [Int64]$env:TITAN_MEMORY_BYTES
if (-not $parent -or -not $diff -or -not $name) { throw 'Missing TITAN_* environment variables' }
if (Get-VM -Name $name -ErrorAction SilentlyContinue) { throw "VM already exists: $name" }
New-VHD -Path $diff -ParentPath $parent -Differencing | Out-Null
New-VM -Name $name -Generation 2 -MemoryStartupBytes $mem -VHDPath $diff | Out-Null
$sw = $env:TITAN_SWITCH_NAME
if ($sw) {
  Get-VMNetworkAdapter -VMName $name | Connect-VMNetworkAdapter -SwitchName $sw
}
"#;

/// Runs the M1 PowerShell pipeline (blocking).
pub fn provision(plan: &VmProvisionPlan) -> Result<()> {
    let diff_path = plan.differencing_vhdx_path();
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(PS_SCRIPT)
        .env("TITAN_PARENT_VHDX", plan.parent_vhdx.trim())
        .env("TITAN_DIFF_VHDX", &diff_path)
        .env("TITAN_VM_NAME", plan.vm_name.trim())
        .env("TITAN_MEMORY_BYTES", plan.memory_bytes.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(sw) = plan.switch_name.as_ref() {
        cmd.env("TITAN_SWITCH_NAME", sw.trim());
    }

    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;

    if out.status.success() {
        return Ok(());
    }

    Err(Error::HyperVRejected {
        message: format_process_failure(out.status.code(), &out.stdout, &out.stderr),
    })
}

const PS_START_VM: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$name = $env:TITAN_VM_NAME
if (-not $name) { throw 'Missing TITAN_VM_NAME environment variable' }
Start-VM -Name $name
"#;

const PS_STOP_VM: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$name = $env:TITAN_VM_NAME
if (-not $name) { throw 'Missing TITAN_VM_NAME environment variable' }
Stop-VM -Name $name -Force
"#;

/// Starts or stops an existing VM by name (blocking).
const PS_LIST_VM: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
Get-VM | ForEach-Object { [Console]::WriteLine(("{0}|{1}" -f $_.Name, $_.State)) }
"#;

fn list_vms_parse_stdout(text: &str) -> Vec<(String, VmPowerState)> {
    let mut vms = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((name, state_raw)) = line.split_once('|') else {
            continue;
        };
        let state = match state_raw.trim() {
            "Running" => VmPowerState::Running,
            "Off" => VmPowerState::Off,
            "Paused" => VmPowerState::Paused,
            _ => VmPowerState::Unknown,
        };
        vms.push((name.trim().to_string(), state));
    }
    vms
}

fn list_vms_powershell_stdout() -> Result<String> {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(PS_LIST_VM)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;

    if !out.status.success() {
        return Err(Error::HyperVRejected {
            message: format_process_failure(out.status.code(), &out.stdout, &out.stderr),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Lists `(name, power_state)` via Hyper-V PowerShell (blocking).
pub fn list_vms() -> Result<Vec<(String, VmPowerState)>> {
    if !hyperv_ps_module_available_blocking() {
        tracing::warn!(
            "list_vms: Hyper-V module not available; returning empty VM list for control plane"
        );
        return Ok(Vec::new());
    }
    let text = list_vms_powershell_stdout()?;
    Ok(list_vms_parse_stdout(&text))
}

const PS_VM_EXISTS: &str = r#"
$ErrorActionPreference = 'Stop'
if (-not (Get-Module -ListAvailable -Name Hyper-V)) {
  throw 'Hyper-V PowerShell module not available. Enable the Hyper-V role.'
}
Import-Module Hyper-V
$name = $env:TITAN_VM_NAME
if (-not $name) { throw 'Missing TITAN_VM_NAME environment variable' }
$vm = Get-VM -Name $name -ErrorAction SilentlyContinue
if ($vm) { [Console]::WriteLine('1') } else { [Console]::WriteLine('0') }
"#;

/// Returns whether a VM with this name exists (blocking).
pub fn vm_exists(vm_name: &str) -> Result<bool> {
    if !hyperv_ps_module_available_blocking() {
        return Ok(false);
    }
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(PS_VM_EXISTS)
        .env("TITAN_VM_NAME", vm_name.trim())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;

    if !out.status.success() {
        return Err(Error::HyperVRejected {
            message: format_process_failure(out.status.code(), &out.stdout, &out.stderr),
        });
    }

    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(text == "1")
}

pub fn vm_power(vm_name: &str, start: bool) -> Result<()> {
    let script = if start { PS_START_VM } else { PS_STOP_VM };
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script)
        .env("TITAN_VM_NAME", vm_name.trim())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;

    if out.status.success() {
        return Ok(());
    }

    Err(Error::HyperVRejected {
        message: format_process_failure(out.status.code(), &out.stdout, &out.stderr),
    })
}

fn format_process_failure(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> String {
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
