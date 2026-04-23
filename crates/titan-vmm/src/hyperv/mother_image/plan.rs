//! JSON planning checklist for mother-image spoof work.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use titan_common::{Error, Result};

#[derive(Debug, Serialize)]
struct PlanDocument {
    schema_version: u32,
    vm_template: String,
    steps: Vec<PlanStep>,
}

#[derive(Debug, Serialize)]
struct PlanStep {
    id: &'static str,
    status: &'static str,
    risk_level: &'static str,
    requires_vm_offline: bool,
    rollback_hint: &'static str,
    detail: String,
}

fn default_plan_steps() -> Vec<PlanStep> {
    vec![
        PlanStep {
            id: "review_eula",
            status: "manual",
            risk_level: "operator",
            requires_vm_offline: false,
            rollback_hint: "N/A",
            detail: "Review game EULA and local law before automating identity changes.".into(),
        },
        PlanStep {
            id: "sysprep_golden",
            status: "manual",
            risk_level: "operator",
            requires_vm_offline: true,
            rollback_hint: "Restore golden from backup",
            detail: "Golden image: Sysprep / generalize per your deployment guide.".into(),
        },
        PlanStep {
            id: "hyperv_checkpoint_policy",
            status: "automatable",
            risk_level: "low",
            requires_vm_offline: false,
            rollback_hint: "Set-VM -CheckpointType Production",
            detail:
                "Optional: Set-VM -CheckpointType Disabled (VmSpoofProfile.disable_checkpoints)."
                    .into(),
        },
        PlanStep {
            id: "hyperv_processor_count",
            status: "automatable",
            risk_level: "medium",
            requires_vm_offline: true,
            rollback_hint: "Set-VM -ProcessorCount to previous value",
            detail: "Optional: Set-VM -ProcessorCount (VmSpoofProfile.processor_count).".into(),
        },
        PlanStep {
            id: "hyperv_network_mac",
            status: "automatable",
            risk_level: "low",
            requires_vm_offline: false,
            rollback_hint: "Set static/dynamic MAC back from audit JSON",
            detail: "Optional: dynamic MAC on synthetic adapters (VmSpoofProfile.dynamic_mac)."
                .into(),
        },
        PlanStep {
            id: "hyperv_network_vlan",
            status: "automatable",
            risk_level: "low",
            requires_vm_offline: false,
            rollback_hint: "Remove-VMNetworkAdapterVlanConfiguration or restore prior VLAN id",
            detail: "Optional: access VLAN on synthetic NICs (VmSpoofProfile.vlan_id_access)."
                .into(),
        },
        PlanStep {
            id: "hyperv_firmware_vtpm",
            status: "automatable",
            risk_level: "medium",
            requires_vm_offline: false,
            rollback_hint: "Disable-VMTPM / restore firmware snapshot",
            detail: "Optional: Enable-VMTPM when supported (VmSpoofProfile.enable_vtpm).".into(),
        },
        PlanStep {
            id: "guest_identity_phase2a",
            status: "planned",
            risk_level: "operator",
            requires_vm_offline: false,
            rollback_hint: "Restore guest snapshot",
            detail: "Phase 2A: guest agent / artifact hooks (VmSpoofProfile.guest_identity_tag)."
                .into(),
        },
    ]
}

/// Writes a JSON checklist for operators (audit trail). Does **not** imply all steps ran.
pub fn plan_mother_image_spoof(vm_template: &str, out_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(out_dir).map_err(Error::Io)?;
    let doc = PlanDocument {
        schema_version: 2,
        vm_template: vm_template.to_string(),
        steps: default_plan_steps(),
    };
    let path = out_dir.join(format!(
        "mother-plan-{}.json",
        vm_template.trim().replace('/', "_")
    ));
    let json = serde_json::to_vec_pretty(&doc).map_err(|e| Error::HyperVRejected {
        message: format!("serialize plan: {e}"),
    })?;
    fs::write(&path, json).map_err(Error::Io)?;
    Ok(path)
}
