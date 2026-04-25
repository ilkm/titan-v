pub(crate) struct BatchReport {
    pub succeeded: u32,
    pub failures: Vec<String>,
}

#[cfg(windows)]
fn batch_power_preflight_windows(start: bool) -> Option<BatchReport> {
    if !titan_vmm::hyperv::gpu_pv::hyperv_ps_module_available_blocking() {
        tracing::warn!(
            start,
            "batch_power skipped: Hyper-V PowerShell module not available"
        );
        return Some(BatchReport {
            succeeded: 0,
            failures: vec![
                "Hyper-V PowerShell module not available (enable the Hyper-V role).".into(),
            ],
        });
    }
    None
}

#[cfg(target_os = "linux")]
fn batch_power_preflight_linux(start: bool) -> Option<BatchReport> {
    if !titan_vmm::platform_vm::linux_virsh_available_blocking() {
        tracing::warn!(start, "batch_power skipped: virsh not on PATH");
        return Some(BatchReport {
            succeeded: 0,
            failures: vec![
                "virsh not available on PATH (install libvirt-client / virt-manager client tools)."
                    .into(),
            ],
        });
    }
    None
}

#[cfg(all(not(windows), not(target_os = "linux")))]
fn batch_power_preflight_other(start: bool) -> Option<BatchReport> {
    tracing::warn!(start, "batch_power: macOS domain power not implemented");
    Some(BatchReport {
        succeeded: 0,
        failures: vec![
            "VM batch power is not implemented on macOS yet (Virtualization.framework path pending)."
                .into(),
        ],
    })
}

fn batch_power_preflight(start: bool) -> Option<BatchReport> {
    #[cfg(windows)]
    {
        return batch_power_preflight_windows(start);
    }
    #[cfg(target_os = "linux")]
    {
        return batch_power_preflight_linux(start);
    }
    #[cfg(all(not(windows), not(target_os = "linux")))]
    {
        batch_power_preflight_other(start)
    }
}

pub(crate) fn batch_power(start: bool, vm_names: &[String]) -> BatchReport {
    if !vm_names.iter().any(|n| !n.trim().is_empty()) {
        return BatchReport {
            succeeded: 0,
            failures: Vec::new(),
        };
    }
    if let Some(r) = batch_power_preflight(start) {
        return r;
    }
    let mut succeeded = 0u32;
    let mut failures = Vec::new();
    for name in vm_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let r = titan_vmm::platform_vm::domain_set_power_blocking(trimmed, start);
        match r {
            Ok(()) => succeeded += 1,
            Err(e) => failures.push(format!("{trimmed}: {e}")),
        }
    }
    BatchReport {
        succeeded,
        failures,
    }
}
