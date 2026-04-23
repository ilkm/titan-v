//! Mother-image and low-risk **guest-visible** tweaks on existing VMs (not offline hive surgery).

mod apply;
#[cfg(windows)]
mod audit;
mod plan;
#[cfg(windows)]
mod probe;
#[cfg(windows)]
mod ps;
mod step_apply;
mod vm_power;

pub use apply::{
    apply_host_spoof_profile, apply_host_spoof_profile_with_options, apply_network_spoof_low_risk,
};
pub use plan::plan_mother_image_spoof;
pub use step_apply::apply_spoof_step;
pub use vm_power::get_vm_power_state_blocking;

#[cfg(windows)]
pub use probe::probe_spoof_host_caps_blocking;

#[cfg(not(windows))]
pub fn probe_spoof_host_caps_blocking() -> titan_common::HypervSpoofHostCaps {
    titan_common::HypervSpoofHostCaps::default()
}

/// Back-compat: true when network adapter spoof cmdlets exist.
pub fn hardware_spoof_cmdlets_available_blocking() -> bool {
    probe_spoof_host_caps_blocking().network_identity
}
