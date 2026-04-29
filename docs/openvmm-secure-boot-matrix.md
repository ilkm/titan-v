# OpenVMM / Secure Boot configuration matrix (Phase 2+)

**Scope:** this matrix applies when the **host OS** runs `titan-host` and delegates VM / paravisor workloads to **[OpenVMM](https://openvmm.dev/)** (or an OpenVMM-backed sidecar), rather than maintaining an in-repo, from-scratch Hyper-V automation stack.

This document aligns `need.md` “方案 B” host vs guest policies with engineering milestones. It is **not** a guarantee of anti-cheat or EULA compliance.

## Roles

- **Host OS**: runs `titan-host`, **OpenVMM** (library or dedicated process per integration design), and optionally a **custom kernel driver** (Phase 2+, separate deliverable).
- **Guest VM**: runs the workload (e.g. game). May use **vTPM** and **guest Secure Boot** independent of host policy; concrete combinations depend on OpenVMM build features and guest firmware (e.g. UEFI / IGVM) in use.

## Matrix (simplified)

| Host Secure Boot | Custom host driver | Guest Secure Boot | Guest vTPM | Notes |
|------------------|--------------------|-------------------|------------|--------|
| On | Not loadable without attestation signing | On/Off per VM template | Optional | Typical corporate / secure baseline. |
| Off | Test-signed / development load | On | Optional | Common lab layout for **driver bring-up** only. |
| Off | Production-signed driver | On | Optional | Requires full signing + release process; **not** in Phase 1. |

## `titan-host` scope

- **Phase 1.x**: control plane and capability probes; VM lifecycle and policy steps are **delegated** to OpenVMM integration (RPC / subprocess / narrow adapter as designed), not reimplemented as a full Titan-owned VMM.
- **Phase 2+**: user-mode service talks to a driver over a **defined IPC**; `Capabilities` must reflect **probed** driver presence, not configuration intent alone.

## Offline hive / disk images (Phase 2B)

Editing offline registry hives on mounted guest disks remains a **storage / image** concern independent of whether the runtime VMM is OpenVMM or legacy Hyper-V tooling. When implementing mount/edit pipelines, use crate `titan-offline-spoof` with feature `offline-hive` if that crate is present in the workspace. Default CI builds **must not** require elevated mounts.
