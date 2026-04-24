//! Virtual machine monitor abstraction.
//!
//! **Platforms**: [`hyperv`] for Windows (M1 provisioning + future VMBus paths). [`kvm`] for
//! Linux/KVM + virtio (placeholder). [`hvf`] for Apple host virtualization (placeholder).
//! [`platform_vm`] routes ListVms / domain power for `titan-host`.

#![forbid(unsafe_code)]

pub mod hvf;
pub mod hyperv;
pub mod kvm;
pub mod platform_vm;

mod traits;

pub use traits::{InjectInput, PowerControl, ReadMemory};
