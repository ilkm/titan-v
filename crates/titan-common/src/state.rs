//! Shared operational state for center ↔ host coordination (transport-agnostic).

use serde::{Deserialize, Serialize};

/// VM power state as seen by orchestration (not necessarily the hypervisor's full enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VmPowerState {
    #[default]
    Unknown,
    Off,
    Running,
    Paused,
}

/// Host node availability for scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    #[default]
    Unknown,
    Online,
    Offline,
    Degraded,
}
