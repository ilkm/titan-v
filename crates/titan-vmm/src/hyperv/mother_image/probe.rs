//! Host-side Hyper-V cmdlet surface probes.

use std::process::{Command, Stdio};

use titan_common::HypervSpoofHostCaps;

#[derive(Debug, serde::Deserialize)]
struct SpoofProbeJson {
    network_identity: bool,
    vm_checkpoint_policy: bool,
    vm_processor_count: bool,
    vm_vlan_config: bool,
    vm_expose_virtualization_extensions: bool,
    vm_firmware_secure_boot: bool,
    vm_vtpm: bool,
}

/// Probes which host-side spoof PowerShell surfaces exist (single short PS invocation on Windows).
pub fn probe_spoof_host_caps_blocking() -> HypervSpoofHostCaps {
    const PS_PROBE: &str = r#"
$ErrorActionPreference = 'Stop'
$n=$false;$c=$false;$p=$false;$v=$false;$e=$false;$f=$false;$t=$false
try {
  Import-Module Hyper-V -ErrorAction Stop
  if ((Get-Command Set-VMNetworkAdapter -EA SilentlyContinue) -and (Get-Command Get-VMNetworkAdapter -EA SilentlyContinue)) { $n = $true }
  if (Get-Command Set-VM -EA SilentlyContinue) { $c = $true; $p = $true }
  if (Get-Command Set-VMNetworkAdapterVlanConfiguration -EA SilentlyContinue) { $v = $true }
  if (Get-Command Set-VMProcessor -EA SilentlyContinue) { $e = $true }
  if (Get-Command Set-VMFirmware -EA SilentlyContinue) { $f = $true }
  if ((Get-Command Enable-VMTPM -EA SilentlyContinue) -or (Get-Command Disable-VMTPM -EA SilentlyContinue)) { $t = $true }
} catch {}
@{ network_identity=$n; vm_checkpoint_policy=$c; vm_processor_count=$p; vm_vlan_config=$v; vm_expose_virtualization_extensions=$e; vm_firmware_secure_boot=$f; vm_vtpm=$t } | ConvertTo-Json -Compress
"#;
    let out = match Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(PS_PROBE)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return HypervSpoofHostCaps::default(),
    };
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    serde_json::from_str::<SpoofProbeJson>(&line)
        .map(|j| HypervSpoofHostCaps {
            network_identity: j.network_identity,
            vm_checkpoint_policy: j.vm_checkpoint_policy,
            vm_processor_count: j.vm_processor_count,
            vm_vlan_config: j.vm_vlan_config,
            vm_expose_virtualization_extensions: j.vm_expose_virtualization_extensions,
            vm_firmware_secure_boot: j.vm_firmware_secure_boot,
            vm_vtpm: j.vm_vtpm,
        })
        .unwrap_or_default()
}
