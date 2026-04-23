//! User-mode ↔ kernel driver contract for VMBus HID and related channels (Phase 4).
//!
//! The repository does not ship a signed kernel driver; this crate holds **interfaces only**
//! so `titan-host` and `titan-vmm` can converge on one abstraction.
//!
//! **Guest agent path**: cooperative in-guest TCP/JSON (see `titan_vmm::hyperv::guest_agent`) is
//! **not** VMBus and must not be confused with [`VmbusHidChannel`], which represents a future
//! kernel-mode HID channel.

#![forbid(unsafe_code)]

/// Recoverable driver-side failures (no secrets in `Display`).
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("rejected: {0}")]
    Rejected(String),
}

pub type DriverResult<T> = std::result::Result<T, DriverError>;

/// HID-style injection over VMBus (implemented by a future driver + user-mode service pair).
pub trait VmbusHidChannel: Send + Sync {
    fn inject_mouse_move(&self, vm_id: &str, x: u32, y: u32) -> DriverResult<()>;
}

/// Cooperative guest agent (TCP/JSON) — **user-mode only**; implemented in `titan-vmm`, not here.
pub trait GuestAgentChannel: Send + Sync {
    fn agent_mouse_move(&self, vm_id: &str, x: u32, y: u32) -> DriverResult<()>;
}

/// Placeholder until a real kernel channel exists.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopVmbusHid;

impl VmbusHidChannel for NoopVmbusHid {
    fn inject_mouse_move(&self, _vm_id: &str, _x: u32, _y: u32) -> DriverResult<()> {
        Err(DriverError::NotImplemented("VMBus HID inject"))
    }
}

/// Placeholder guest-agent channel (wire-up lives in `titan-vmm::hyperv::HypervHostRuntime`).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopGuestAgentChannel;

impl GuestAgentChannel for NoopGuestAgentChannel {
    fn agent_mouse_move(&self, _vm_id: &str, _x: u32, _y: u32) -> DriverResult<()> {
        Err(DriverError::NotImplemented("guest agent TCP"))
    }
}
