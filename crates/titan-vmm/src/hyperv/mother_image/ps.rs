//! PowerShell job wrappers and Hyper-V cmdlet script bodies.

use std::process::{Command, Stdio};

use titan_common::{Error, Result};

pub(super) const PS_JOB_TIMEOUT_SEC: u32 = 120;
const PS_OUTPUT_MAX: usize = 4096;

pub(super) fn truncate_output(s: &str) -> String {
    if s.len() <= PS_OUTPUT_MAX {
        s.to_string()
    } else {
        format!("{}…(truncated)", &s[..PS_OUTPUT_MAX])
    }
}

pub(super) fn format_ps(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = truncate_output(String::from_utf8_lossy(stdout).trim());
    let stderr = truncate_output(String::from_utf8_lossy(stderr).trim());
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

pub(super) fn run_ps_job(vm: &str, script: &str) -> Result<()> {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script)
        .env("TITAN_VM_NAME", vm.trim())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = cmd.output().map_err(|e| Error::HyperVRejected {
        message: format!("failed to run powershell.exe: {e}"),
    })?;
    if out.status.success() {
        return Ok(());
    }
    Err(Error::HyperVRejected {
        message: format_ps(out.status.code(), &out.stdout, &out.stderr),
    })
}

fn job_wrap(inner: &str, to: u32) -> String {
    format!(
        r#"$ErrorActionPreference='Stop'
$vm = [string]$env:TITAN_VM_NAME
if (-not $vm) {{ throw 'Missing TITAN_VM_NAME' }}
$job = Start-Job -ScriptBlock {{
  {inner}
}} -ArgumentList $vm
if (-not (Wait-Job $job -Timeout {to})) {{
  Stop-Job $job -Force -ErrorAction SilentlyContinue
  Remove-Job $job -Force -ErrorAction SilentlyContinue
  throw 'titan:JobTimeout'
}}
try {{
  Receive-Job $job -ErrorAction Stop | Out-Null
}} finally {{
  Remove-Job $job -Force -ErrorAction SilentlyContinue
}}
"#,
        inner = inner,
        to = to
    )
}

pub(super) fn dynamic_mac_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  if (-not (Get-Module -ListAvailable -Name Hyper-V)) { throw 'Hyper-V module missing' }
  Import-Module Hyper-V
  Get-VMNetworkAdapter -VMName $vm | Set-VMNetworkAdapter -DynamicMacAddress On",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn checkpoint_disabled_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  if (-not (Get-Module -ListAvailable -Name Hyper-V)) { throw 'Hyper-V module missing' }
  Import-Module Hyper-V
  Set-VM -VMName $vm -CheckpointType Disabled",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn processor_count_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  if (-not (Get-Module -ListAvailable -Name Hyper-V)) { throw 'Hyper-V module missing' }
  Import-Module Hyper-V
  $pc = [int]$env:TITAN_PROCESSOR_COUNT
  Set-VM -VMName $vm -ProcessorCount $pc",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn expose_ve_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  Import-Module Hyper-V
  $on = $env:TITAN_EXPOSE_VE -eq 'true'
  Set-VMProcessor -VMName $vm -ExposeVirtualizationExtensions:$on",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn vlan_access_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  Import-Module Hyper-V
  $vid = [int]$env:TITAN_VLAN_ID
  $na = Get-VMNetworkAdapter -VMName $vm | Select-Object -First 1
  if (-not $na) { throw 'no network adapter' }
  Set-VMNetworkAdapterVlanConfiguration -VMNetworkAdapter $na -Access -VlanId $vid",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn static_mac_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  Import-Module Hyper-V
  $line = (Get-Content -Path $env:TITAN_MAC_POOL_FILE -TotalCount 1).Trim()
  if (-not $line) { throw 'empty mac pool file' }
  $na = Get-VMNetworkAdapter -VMName $vm | Select-Object -First 1
  if (-not $na) { throw 'no network adapter' }
  Set-VMNetworkAdapter -VMNetworkAdapter $na -StaticMacAddress $line",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn secure_boot_template_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  Import-Module Hyper-V
  $t = [string]$env:TITAN_SB_TEMPLATE
  Set-VMFirmware -VMName $vm -SecureBootTemplate $t",
        PS_JOB_TIMEOUT_SEC,
    )
}

pub(super) fn enable_vtpm_job_script() -> String {
    job_wrap(
        r"param($vm)
  $ErrorActionPreference='Stop'
  Import-Module Hyper-V
  if ($env:TITAN_VTPM_ENABLE -eq 'true') { Enable-VMTPM -VMName $vm } else { Disable-VMTPM -VMName $vm -ErrorAction SilentlyContinue }",
        PS_JOB_TIMEOUT_SEC,
    )
}
