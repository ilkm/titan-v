//! Virtual machine monitor abstraction.
//!
//! **Platforms**: [`hyperv`] for Windows (M1 provisioning + future VMBus paths). [`kvm`] for
//! Linux/KVM + virtio (placeholder). [`mac`] for Apple `Virtualization.framework` on macOS
//! (placeholder).

#![forbid(unsafe_code)]

pub mod hyperv;
pub mod kvm;
pub mod mac;

mod traits;

pub use traits::{InjectInput, PowerControl, ReadMemory};
