//! Virtual machine monitor abstraction.
//!
//! **Production path**: [`hyperv`] on Windows (M1 provisioning + future VMBus paths).
//! [`platform_vm`] routes ListVms / domain power for `titan-host` on Windows to `hyperv`.

#![forbid(unsafe_code)]

pub mod hyperv;
pub mod platform_vm;

mod traits;

pub use traits::{InjectInput, PowerControl, ReadMemory};
