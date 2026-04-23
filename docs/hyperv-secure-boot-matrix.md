# Hyper-V / Secure Boot configuration matrix (Phase 2+)

This document aligns `need.md` “方案 B” host vs guest policies with engineering milestones. It is **not** a guarantee of anti-cheat or EULA compliance.

## Roles

- **Host OS**: runs `titan-host`, Hyper-V, and optionally a **custom kernel driver** (Phase 2+, separate deliverable).
- **Guest VM**: runs the workload (e.g. game). May use **vTPM** and **guest Secure Boot** independent of host policy.

## Matrix (simplified)

| Host Secure Boot | Custom host driver | Guest Secure Boot | Guest vTPM | Notes |
|------------------|--------------------|-------------------|------------|--------|
| On | Not loadable without attestation signing | On/Off per VM template | Optional | Typical corporate / secure baseline. |
| Off | Test-signed / development load | On | Optional | Common lab layout for **driver bring-up** only. |
| Off | Production-signed driver | On | Optional | Requires full signing + release process; **not** in Phase 1. |

## `titan-host` scope

- **Phase 1.x**: PowerShell-only VM configuration (NIC MAC policy, checkpoints, processor count). No host driver.
- **Phase 2+**: User-mode service talks to a driver over a **defined IPC**; `Capabilities` must reflect **probed** driver presence, not configuration intent alone.

## Offline hive / VHDX (Phase 2B)

Use crate `titan-offline-spoof` with feature `offline-hive` when implementing mount/edit pipelines. Default CI builds **must not** require elevated mounts.
