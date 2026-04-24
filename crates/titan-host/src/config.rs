//! TOML configuration for batch VM provisioning.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use titan_common::{VmIdentityProfile, VmProvisionPlan, VmSpoofProfile};

/// Maximum VMs per `[[vm_group]]` (safety bound).
pub const MAX_VM_GROUP_COUNT: u32 = 64;

/// Expandable template: `vm_name` = `name_prefix` + zero-padded index.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VmGroup {
    pub parent_vhdx: String,
    pub diff_dir: String,
    pub name_prefix: String,
    pub count: u32,
    pub memory_bytes: u64,
    pub generation: u8,
    pub switch_name: Option<String>,
    /// Shared by all VMs expanded from this group (optional).
    #[serde(default)]
    pub gpu_partition_instance_path: Option<String>,
    #[serde(default = "default_auto_start_after_provision")]
    pub auto_start_after_provision: bool,
    /// Host-side spoof profile applied to every VM in this group after create.
    #[serde(default)]
    pub spoof: VmSpoofProfile,
    #[serde(default)]
    pub identity: VmIdentityProfile,
}

fn default_auto_start_after_provision() -> bool {
    true
}

/// Root config file for batch VM provisioning ([`crate::batch::run_provision`]).
#[derive(Debug, Deserialize)]
pub struct HostConfigFile {
    /// Max wall-clock time per VM for the outer async wait (see `HostConfigFile::timeout`).
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub vm: Vec<VmProvisionPlan>,
    #[serde(default)]
    pub vm_group: Vec<VmGroup>,
}

fn default_timeout_secs() -> u64 {
    600
}

fn index_pad_width(count: u32) -> usize {
    if count <= 1 {
        return 2;
    }
    ((count - 1).ilog10() as usize + 1).max(2)
}

/// Merges explicit `vm` entries with expanded `vm_group` templates; checks uniqueness (same rules as TOML config).
pub fn expand_vm_plans(
    vm: &[VmProvisionPlan],
    vm_group: &[VmGroup],
) -> anyhow::Result<Vec<VmProvisionPlan>> {
    let mut out: Vec<VmProvisionPlan> = vm.to_vec();
    for g in vm_group {
        if g.count == 0 || g.count > MAX_VM_GROUP_COUNT {
            anyhow::bail!(
                "vm_group count must be 1..={MAX_VM_GROUP_COUNT} (got {})",
                g.count
            );
        }
        let width = index_pad_width(g.count);
        for i in 0..g.count {
            let vm_name = format!("{}{:0width$}", g.name_prefix, i, width = width);
            let plan = VmProvisionPlan {
                parent_vhdx: g.parent_vhdx.clone(),
                diff_dir: g.diff_dir.clone(),
                vm_name,
                memory_bytes: g.memory_bytes,
                generation: g.generation,
                switch_name: g.switch_name.clone(),
                gpu_partition_instance_path: g.gpu_partition_instance_path.clone(),
                auto_start_after_provision: g.auto_start_after_provision,
                spoof: g.spoof.clone(),
                identity: g.identity.clone(),
            };
            plan.validate()
                .map_err(|e| anyhow::anyhow!("vm_group entry invalid: {e}"))?;
            out.push(plan);
        }
    }

    let mut seen = HashSet::new();
    for p in &out {
        if !seen.insert(p.vm_name.clone()) {
            anyhow::bail!("duplicate vm_name after expansion: {}", p.vm_name);
        }
    }

    Ok(out)
}

impl HostConfigFile {
    /// Loads and parses a TOML config from disk.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("read host config {}", path.display()))?;
        Self::parse(&raw).with_context(|| format!("parse host config TOML ({})", path.display()))
    }

    /// Parses TOML from memory (for tests and tooling).
    pub fn parse(raw: &str) -> anyhow::Result<Self> {
        toml::from_str(raw).context("parse host config TOML")
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs.max(1))
    }

    /// Merges explicit `[[vm]]` entries with expanded `[[vm_group]]` templates; checks uniqueness.
    pub fn expanded_vm_plans(&self) -> anyhow::Result<Vec<VmProvisionPlan>> {
        expand_vm_plans(&self.vm, &self.vm_group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let raw = r#"
timeout_secs = 120

[[vm]]
parent_vhdx = "D:\\parent.vhdx"
diff_dir = "D:\\diffs"
vm_name = "game-01"
memory_bytes = 2147483648
generation = 2
switch_name = "External"
"#;
        let cfg = HostConfigFile::parse(raw).unwrap();
        assert_eq!(cfg.timeout_secs, 120);
        let plans = cfg.expanded_vm_plans().unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].vm_name, "game-01");
        plans[0].validate().unwrap();
    }

    #[test]
    fn expands_vm_group_three() {
        let raw = r#"
[[vm_group]]
parent_vhdx = "D:\\p.vhdx"
diff_dir = "D:\\d"
name_prefix = "game-"
count = 3
memory_bytes = 1073741824
generation = 2
"#;
        let cfg = HostConfigFile::parse(raw).unwrap();
        let via_fn = expand_vm_plans(&cfg.vm, &cfg.vm_group).unwrap();
        let plans = cfg.expanded_vm_plans().unwrap();
        assert_eq!(via_fn, plans);
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].vm_name, "game-00");
        assert_eq!(plans[1].vm_name, "game-01");
        assert_eq!(plans[2].vm_name, "game-02");
    }

    #[test]
    fn rejects_duplicate_after_expansion() {
        let raw = r#"
[[vm]]
parent_vhdx = "D:\\p.vhdx"
diff_dir = "D:\\d"
vm_name = "game-00"
memory_bytes = 1073741824
generation = 2

[[vm_group]]
parent_vhdx = "D:\\p.vhdx"
diff_dir = "D:\\d"
name_prefix = "game-"
count = 2
memory_bytes = 1073741824
generation = 2
"#;
        let cfg = HostConfigFile::parse(raw).unwrap();
        let err = cfg.expanded_vm_plans().unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn parses_vm_with_gpu_and_auto_start_false() {
        let raw = r#"
[[vm]]
parent_vhdx = "D:\\parent.vhdx"
diff_dir = "D:\\diffs"
vm_name = "g1"
memory_bytes = 1073741824
generation = 2
gpu_partition_instance_path = "PCI\\VEN_1234&DEV_5678"
auto_start_after_provision = false
"#;
        let cfg = HostConfigFile::parse(raw).unwrap();
        let plans = cfg.expanded_vm_plans().unwrap();
        assert_eq!(
            plans[0].gpu_partition_instance_path.as_deref(),
            Some("PCI\\VEN_1234&DEV_5678")
        );
        assert!(!plans[0].auto_start_after_provision);
    }

    #[test]
    fn parses_phase1_example_fixture() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/host.phase1.example.toml");
        let cfg = HostConfigFile::load(&path).unwrap();
        let plans = cfg.expanded_vm_plans().unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].vm_name, "demo-01");
        assert!(plans[0].auto_start_after_provision);
        assert!(plans[0].gpu_partition_instance_path.is_none());
    }

    #[test]
    fn parses_vm_inline_spoof_profile() {
        let raw = r#"
timeout_secs = 60

[[vm]]
parent_vhdx = "D:\\parent.vhdx"
diff_dir = "D:\\diffs"
vm_name = "sp-01"
memory_bytes = 1073741824
generation = 2
spoof = { dynamic_mac = false, disable_checkpoints = true, processor_count = 4 }
"#;
        let cfg = HostConfigFile::parse(raw).unwrap();
        let plans = cfg.expanded_vm_plans().unwrap();
        assert_eq!(plans.len(), 1);
        assert!(!plans[0].spoof.dynamic_mac);
        assert!(plans[0].spoof.disable_checkpoints);
        assert_eq!(plans[0].spoof.processor_count, Some(4));
    }
}
