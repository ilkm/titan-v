//! Messages from the background network thread to the UI thread.

use titan_common::VmBrief;

pub enum NetUiMsg {
    Caps {
        summary: String,
    },
    VmInventory(Vec<VmBrief>),
    BatchStop {
        succeeded: u32,
        failures: Vec<String>,
    },
    BatchStart {
        succeeded: u32,
        failures: Vec<String>,
    },
    SpoofApply {
        dry_run: bool,
        steps: Vec<String>,
        notes: String,
    },
    GuestAgentReg {
        vm_name: String,
    },
    Error(String),
}
