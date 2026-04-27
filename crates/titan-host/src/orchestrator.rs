//! Post-M1 pipeline: GPU-PV, spoofing, stream precheck, and cooperative guest input ([`HypervHostRuntime`]).

use titan_common::{
    GpuPartitioner, HardwareSpoofer, Result, StreamEncoder, VmIdentityProfile, VmProvisionPlan,
    VmbusInput,
};
use titan_vmm::hyperv::{
    mother_image, HypervBackend, HypervGpuPartitioner, HypervHardwareSpoofer, HypervHostRuntime,
    HypervStreamPrecheck,
};
use titan_vmm::PowerControl;

/// Holds concrete Hyper-V subsystems after differencing-disk provisioning.
pub struct Orchestrator {
    /// Power + provisioning entry (`New-VM` / `Start-VM` paths).
    pub hyperv: HypervBackend,
    /// Guest agent bindings + read/inject + VMBusInput delegation.
    pub cooperative: HypervHostRuntime,
    pub hardware: HypervHardwareSpoofer,
    pub gpu: HypervGpuPartitioner,
    pub stream: HypervStreamPrecheck,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self {
            hyperv: HypervBackend,
            cooperative: HypervHostRuntime::default(),
            hardware: HypervHardwareSpoofer,
            gpu: HypervGpuPartitioner,
            stream: HypervStreamPrecheck,
        }
    }
}

impl Orchestrator {
    /// Intended order: provision → hardware → GPU → power → stream precheck → input.
    pub fn log_post_m1_pipeline() {
        tracing::debug!(
            target: "titan_host::orchestrator",
            hardware = std::any::type_name::<HypervHardwareSpoofer>(),
            gpu = std::any::type_name::<HypervGpuPartitioner>(),
            stream = std::any::type_name::<HypervStreamPrecheck>(),
            cooperative = std::any::type_name::<HypervHostRuntime>(),
            "post-M1 pipeline types"
        );
    }

    fn log_post_step(vm: &str, step: &'static str, ok: bool, err: Option<String>) {
        if ok {
            tracing::info!(target: "titan_host::orchestrator", %vm, step, ok, "post-provision step");
        } else {
            tracing::warn!(
                target: "titan_host::orchestrator",
                %vm,
                step,
                ok,
                error = err.as_deref().unwrap_or(""),
                "post-provision step"
            );
        }
    }

    fn post_maybe_gpu_assign(
        orch: &Self,
        vm: &str,
        plan: &VmProvisionPlan,
        fail_fast: bool,
    ) -> Result<()> {
        let Some(ref path) = plan.gpu_partition_instance_path else {
            return Ok(());
        };
        let p = path.trim();
        if p.is_empty() {
            return Ok(());
        }
        let r = orch.gpu.assign(vm, p);
        match &r {
            Ok(()) => Self::log_post_step(vm, "gpu_assign", true, None),
            Err(e) => Self::log_post_step(vm, "gpu_assign", false, Some(e.to_string())),
        }
        if r.is_err() && fail_fast {
            return r;
        }
        Ok(())
    }

    fn post_maybe_power_start(
        orch: &Self,
        vm: &str,
        plan: &VmProvisionPlan,
        fail_fast: bool,
    ) -> Result<()> {
        if !plan.auto_start_after_provision {
            tracing::info!(target: "titan_host::orchestrator", %vm, step = "power_start", skipped = true, "post-provision: auto_start_after_provision is false");
            return Ok(());
        }
        let r = orch.hyperv.start(vm);
        match &r {
            Ok(()) => Self::log_post_step(vm, "power_start", true, None),
            Err(e) => Self::log_post_step(vm, "power_start", false, Some(e.to_string())),
        }
        if r.is_err() && fail_fast {
            return r;
        }
        Ok(())
    }

    fn post_log_identity_hints(vm: &str, plan: &VmProvisionPlan) {
        if plan.identity != VmIdentityProfile::default() {
            tracing::info!(
                target: "titan_host::orchestrator",
                %vm,
                identity = ?plan.identity,
                "VmIdentityProfile is non-default; align Capabilities / driver bridge with intent"
            );
        }
        let Some(tag) = plan.spoof.guest_identity_tag.as_deref().map(str::trim) else {
            return;
        };
        if tag.is_empty() {
            return;
        }
        tracing::info!(
            target: "titan_host::orchestrator",
            %vm,
            guest_identity_tag = %tag,
            "Phase 2A: run guest `identity_ops` / artifact pull for this tag after agent binds (SetScriptArtifact + agent addr on host)"
        );
    }

    /// After `New-VM` succeeds: hardware → optional GPU-PV → optional start → stream precheck.
    ///
    /// When `fail_fast` is false, failures are logged and later steps still run where applicable.
    pub fn post_provision_after_create(plan: &VmProvisionPlan, fail_fast: bool) -> Result<()> {
        let orch = Self::default();
        let vm = plan.vm_name.trim();

        let r = mother_image::apply_host_spoof_profile(vm, &plan.spoof);
        match &r {
            Ok(()) => Self::log_post_step(vm, "spoof_profile_apply", true, None),
            Err(e) => Self::log_post_step(vm, "spoof_profile_apply", false, Some(e.to_string())),
        }
        if r.is_err() && fail_fast {
            return r;
        }

        Self::post_log_identity_hints(vm, plan);
        Self::post_maybe_gpu_assign(&orch, vm, plan, fail_fast)?;
        Self::post_maybe_power_start(&orch, vm, plan, fail_fast)?;

        let r = orch.stream.start_session(vm);
        match &r {
            Ok(()) => Self::log_post_step(vm, "stream_precheck", true, None),
            Err(e) => Self::log_post_step(vm, "stream_precheck", false, Some(e.to_string())),
        }
        if r.is_err() && fail_fast {
            return r;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn spoofers(&self) -> (&dyn HardwareSpoofer, &dyn GpuPartitioner) {
        (&self.hardware, &self.gpu)
    }

    #[allow(dead_code)]
    pub fn io_stack(&self) -> (&dyn StreamEncoder, &dyn VmbusInput) {
        (&self.stream, &self.cooperative)
    }
}
