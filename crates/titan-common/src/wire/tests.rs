use crate::capabilities::{Capabilities, HypervSpoofHostCaps};
use crate::plan::VmSpoofProfile;
use crate::state::VmPowerState;
use crate::PROTOCOL_VERSION;

use super::*;

#[test]
fn ping_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    let req = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(req, ControlRequest::Ping));
}

#[test]
fn hello_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::Hello).unwrap();
    let req = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(req, ControlRequest::Hello));
}

#[test]
fn pong_roundtrip() {
    let res = ControlResponse::Pong {
        capabilities: Capabilities::host_control_plane_with_agents(
            false,
            false,
            HypervSpoofHostCaps::default(),
        ),
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out, ControlResponse::Pong { .. }));
}

#[test]
fn list_vms_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::ListVms).unwrap();
    let req = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(req, ControlRequest::ListVms));
}

#[test]
fn vm_list_roundtrip() {
    let res = ControlResponse::VmList {
        vms: vec![VmBrief {
            name: "a".into(),
            state: VmPowerState::Running,
        }],
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out, ControlResponse::VmList { .. }));
}

#[test]
fn hello_ack_roundtrip() {
    let res = ControlResponse::HelloAck {
        capabilities: Capabilities::host_control_plane_with_agents(
            false,
            false,
            HypervSpoofHostCaps::default(),
        ),
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out, ControlResponse::HelloAck { .. }));
}

#[test]
fn bad_magic_rejected() {
    let mut buf = encode_request_frame(&ControlRequest::Hello).unwrap();
    buf[0] = b'X';
    let err = read_request_frame(&mut buf.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::BadMagic));
}

#[test]
fn version_mismatch_rejected() {
    let mut frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    // bump version field in header (offset 8)
    frame[8] = 0xff;
    let err = read_request_frame(&mut frame.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::UnsupportedVersion { .. }));
}

#[test]
fn oversized_len_rejected_without_allocating_huge() {
    let mut hdr = Vec::new();
    hdr.extend_from_slice(&WIRE_MAGIC);
    hdr.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    hdr.extend_from_slice(&(MAX_PAYLOAD_BYTES + 1).to_le_bytes());
    let err = read_request_frame(&mut hdr.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::PayloadTooLarge(_)));
}

#[test]
fn register_guest_agent_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::RegisterGuestAgent {
        vm_name: "vm1".into(),
        guest_agent_addr: "10.0.0.5:9000".into(),
    })
    .unwrap();
    let req = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        req,
        ControlRequest::RegisterGuestAgent { ref vm_name, .. } if vm_name == "vm1"
    ));
}

#[test]
fn guest_agent_register_ack_roundtrip() {
    let res = ControlResponse::GuestAgentRegisterAck {
        vm_name: "vm1".into(),
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out, ControlResponse::GuestAgentRegisterAck { .. }));
}

#[test]
fn apply_spoof_profile_roundtrip() {
    let req = ControlRequest::ApplySpoofProfile {
        vm_name: "vm-a".into(),
        dry_run: true,
        spoof: VmSpoofProfile::default(),
    };
    let frame = encode_request_frame(&req).unwrap();
    let out = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        out,
        ControlRequest::ApplySpoofProfile {
            ref vm_name,
            dry_run: true,
            ..
        } if vm_name == "vm-a"
    ));
}

#[test]
fn apply_spoof_step_roundtrip() {
    let req = ControlRequest::ApplySpoofStep {
        vm_name: "vm-b".into(),
        step_id: "dynamic_mac".into(),
        dry_run: false,
    };
    let frame = encode_request_frame(&req).unwrap();
    let out = read_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        out,
        ControlRequest::ApplySpoofStep {
            ref step_id,
            ..
        } if step_id == "dynamic_mac"
    ));
}

#[test]
fn spoof_apply_ack_roundtrip() {
    let res = ControlResponse::SpoofApplyAck {
        vm_name: "v".into(),
        dry_run: true,
        steps_executed: vec!["a(dry-run)".into()],
        notes: "n".into(),
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out, ControlResponse::SpoofApplyAck { .. }));
}
