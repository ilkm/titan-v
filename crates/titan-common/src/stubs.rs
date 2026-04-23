//! Placeholder traits for GPU-PV, spoofing, streaming, and VMBus input (`need.md` later phases).

use crate::error::{Error, Result};

/// Randomize MAC, disk serial, CPUID exposure (not implemented).
pub trait HardwareSpoofer {
    fn apply(&self, _vm_name: &str) -> Result<()>;
}

/// Assign GPU-PV partition to a VM (not implemented).
pub trait GpuPartitioner {
    fn assign(&self, _vm_name: &str, _partition_id: &str) -> Result<()>;
}

/// Encode VM framebuffer for the center (not implemented).
pub trait StreamEncoder {
    fn start_session(&self, _vm_name: &str) -> Result<()>;
}

/// Inject HID via VMBus (not implemented).
pub trait VmbusInput {
    fn tap(&self, _vm_name: &str, _x: u32, _y: u32) -> Result<()>;
}

/// No-op / stub implementations returning [`Error::NotImplemented`].
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopHardwareSpoofer;

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopGpuPartitioner;

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopStreamEncoder;

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopVmbusInput;

impl HardwareSpoofer for NoopHardwareSpoofer {
    fn apply(&self, _vm_name: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "hardware spoofing",
        })
    }
}

impl GpuPartitioner for NoopGpuPartitioner {
    fn assign(&self, _vm_name: &str, _partition_id: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "GPU-PV partition",
        })
    }
}

impl StreamEncoder for NoopStreamEncoder {
    fn start_session(&self, _vm_name: &str) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "NVENC / streaming",
        })
    }
}

impl VmbusInput for NoopVmbusInput {
    fn tap(&self, _vm_name: &str, _x: u32, _y: u32) -> Result<()> {
        Err(Error::NotImplemented {
            feature: "VMBus HID injection",
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;

    #[test]
    fn noop_hardware_is_not_implemented() {
        let err = NoopHardwareSpoofer.apply("vm-1").unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }

    #[test]
    fn noop_vmbus_is_not_implemented() {
        let err = NoopVmbusInput.tap("vm-1", 10, 20).unwrap_err();
        assert!(matches!(err, Error::NotImplemented { .. }));
    }
}
