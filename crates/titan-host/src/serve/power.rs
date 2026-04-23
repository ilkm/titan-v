use titan_vmm::PowerControl;

pub(crate) struct BatchReport {
    pub succeeded: u32,
    pub failures: Vec<String>,
}

pub(crate) fn batch_power(start: bool, vm_names: &[String]) -> BatchReport {
    let hyperv = titan_vmm::hyperv::HypervBackend;
    let mut succeeded = 0u32;
    let mut failures = Vec::new();
    for name in vm_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let r = if start {
            hyperv.start(trimmed)
        } else {
            hyperv.stop(trimmed)
        };
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
