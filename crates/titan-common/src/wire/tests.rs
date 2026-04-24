use crate::capabilities::{Capabilities, HypervSpoofHostCaps};
use crate::plan::VmSpoofProfile;
use crate::state::VmPowerState;
use std::borrow::Cow;

use crate::PROTOCOL_VERSION;

use super::*;

#[test]
fn ping_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    let req = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert_eq!(req.id, 1);
    assert!(matches!(req.body, ControlRequest::Ping));
}

#[test]
fn hello_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::Hello).unwrap();
    let req = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(req.body, ControlRequest::Hello));
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
    let req = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(req.body, ControlRequest::ListVms));
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
    let err = read_control_request_frame(&mut buf.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::BadMagic));
}

#[test]
fn version_mismatch_rejected() {
    let mut frame = encode_request_frame(&ControlRequest::Ping).unwrap();
    // bump version field in header (offset 8)
    frame[8] = 0xff;
    let err = read_control_request_frame(&mut frame.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::UnsupportedVersion { .. }));
}

#[test]
fn oversized_len_rejected_without_allocating_huge() {
    let mut hdr = Vec::new();
    hdr.extend_from_slice(&WIRE_MAGIC);
    hdr.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    hdr.extend_from_slice(&(MAX_PAYLOAD_BYTES + 1).to_le_bytes());
    let err = read_control_request_frame(&mut hdr.as_slice()).unwrap_err();
    assert!(matches!(err, WireError::PayloadTooLarge(_)));
}

#[test]
fn apply_spoof_profile_roundtrip() {
    let req = ControlRequest::ApplySpoofProfile {
        vm_name: "vm-a".into(),
        dry_run: true,
        spoof: VmSpoofProfile::default(),
    };
    let frame = encode_request_frame(&req).unwrap();
    let out = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        out.body,
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
    let out = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        out.body,
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

#[test]
fn control_host_response_roundtrip() {
    let res = ControlResponse::Pong {
        capabilities: Capabilities::host_control_plane_with_agents(
            false,
            false,
            HypervSpoofHostCaps::default(),
        ),
    };
    let frame =
        encode_control_host_frame(&ControlHostFrame::Response { id: 42, body: res }).unwrap();
    let out = read_control_host_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(
        out,
        ControlHostFrame::Response {
            id: 42,
            body: ControlResponse::Pong { .. }
        }
    ));
}

#[test]
fn host_resource_snapshot_request_roundtrip() {
    let frame = encode_request_frame(&ControlRequest::HostResourceSnapshot).unwrap();
    let out = read_control_request_frame(&mut frame.as_slice()).unwrap();
    assert!(matches!(out.body, ControlRequest::HostResourceSnapshot));
}

#[test]
fn host_resource_snapshot_response_roundtrip() {
    let res = ControlResponse::HostResourceSnapshot {
        stats: HostResourceStats {
            cpu_permille: 123,
            mem_used_bytes: 4_000_000_000,
            mem_total_bytes: 16_000_000_000,
            net_down_bps: 1_048_576,
            net_up_bps: 65_536,
            disk_read_bps: 10_240,
            disk_write_bps: 5_120,
        },
    };
    let frame = encode_response_frame(&res).unwrap();
    let out = read_response_frame(&mut frame.as_slice()).unwrap();
    assert_eq!(out, res);
}

#[test]
fn host_resource_live_push_roundtrip() {
    let push = ControlPush::HostResourceLive {
        stats: HostResourceStats {
            cpu_permille: 500,
            mem_used_bytes: 8,
            mem_total_bytes: 16,
            net_down_bps: 100,
            net_up_bps: 200,
            disk_read_bps: 30,
            disk_write_bps: 40,
        },
    };
    let frame = encode_telemetry_push_frame(&push).unwrap();
    let got = read_telemetry_push_frame(&mut frame.as_slice()).unwrap();
    assert_eq!(got, push);
}

#[test]
fn host_desktop_preview_jpeg_push_roundtrip() {
    let push = ControlPush::HostDesktopPreviewJpeg {
        jpeg_bytes: vec![0xFF, 0xD8, 0xFF, 0xD9],
        width_px: 640,
        height_px: 360,
    };
    assert!(telemetry_push_payload_fits(&push));
    let frame = encode_telemetry_push_frame(&push).unwrap();
    let got = read_telemetry_push_frame(&mut frame.as_slice()).unwrap();
    assert_eq!(got, push);
}

#[test]
fn maybe_zstd_skips_small_payload() {
    let small = vec![0u8; 64];
    let out = crate::maybe_zstd_compress(&small).expect("compress");
    assert!(matches!(out, Cow::Borrowed(_)));
    let big = vec![7u8; 512];
    let out2 = crate::maybe_zstd_compress(&big).expect("compress2");
    assert!(matches!(out2, Cow::Owned(_)));
}

#[test]
fn telemetry_push_roundtrip() {
    let push = ControlPush::HostTelemetry {
        vms: vec![VmBrief {
            name: "x".into(),
            state: VmPowerState::Off,
        }],
        volumes: vec![DiskVolume {
            mount: "C:\\".into(),
            free_bytes: 100,
            total_bytes: 200,
        }],
        content_hint: None,
    };
    let frame = encode_telemetry_push_frame(&push).unwrap();
    let got = read_telemetry_push_frame(&mut frame.as_slice()).unwrap();
    assert_eq!(got, push);
}
