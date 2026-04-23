//! Windows.Graphics.Capture + encoder pipeline (Phase 5).
//!
//! Full `Windows.Graphics.Capture` + NVENC is **not** wired; this module exposes **prechecks**
//! so callers can fail fast before attempting capture work.

/// Placeholder for a per-VM capture session (`Windows.Graphics.Capture` + optional NVENC).
#[derive(Debug, Default)]
pub struct GraphicsCaptureStub;

impl GraphicsCaptureStub {
    #[must_use]
    pub const fn describe() -> &'static str {
        "GraphicsCaptureSession + encoder not wired in this build"
    }

    /// Returns whether the Hyper-V VM exists (Windows). Non-Windows returns `Ok(false)`.
    pub fn vm_exists_precheck(vm_name: &str) -> titan_common::Result<bool> {
        titan_vmm::hyperv::vm_exists_blocking(vm_name)
    }
}
