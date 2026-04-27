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

#[cfg(not(windows))]
fn batch_power_preflight_non_windows(start: bool) -> Option<BatchReport> {
    tracing::warn!(
        start,
        "batch_power skipped: Hyper-V / VM power is only supported on Windows hosts"
    );
    Some(BatchReport {
        succeeded: 0,
        failures: vec![
            "VM batch power requires Windows with Hyper-V (non-Windows stub build).".into(),
        ],
    })
}

fn batch_power_preflight(start: bool) -> Option<BatchReport> {
    #[cfg(windows)]
    {
        return batch_power_preflight_windows(start);
    }
    #[cfg(not(windows))]
    {
        batch_power_preflight_non_windows(start)
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
