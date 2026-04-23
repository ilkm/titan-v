# Requirements traceability (need.md → delivery)

Rows reference [need.md](../need.md). Columns track where behavior is implemented or explicitly deferred.

| need.md theme | Target capability | Code / doc anchor | Test / E2E |
|---------------|-------------------|-------------------|------------|
| 方案 B 环境（宿主 SB / 驱动） | Host secure boot + driver matrix | [hyperv-secure-boot-matrix.md](hyperv-secure-boot-matrix.md), `VmIdentityProfile::host_kernel_driver_expected` | Manual / driver repo CI |
| 方案 B 来宾 vTPM / SB | Guest firmware policy | `VmSpoofProfile::secure_boot_template`, `enable_vtpm`, `mother_image` PS | Windows VM template |
| 伪装 MAC / 磁盘 / CPUID | Layer A+B+C | `VmSpoofProfile`, `mother_image`, guest `identity_echo` / bundle, future driver | Unit + Windows IT |
| VMBus HID 注入 | Layer C | `Capabilities::vmbus_hid`, driver IPC (see `titan-driver-bridge`) | Driver integration |
| WinHv / 窗外读内存 | Layer C + Lua | `Capabilities::winhv_guest_memory`, `titan-host` runtime | Contract tests |
| M2 编排伪装流水线 | Center ↔ host | `ControlRequest::ApplySpoofProfile`, `ApplySpoofStep`, [wire.rs](../crates/titan-common/src/wire.rs) | serve integration tests |
| 母盘 Hive / Sysprep | Layer B | [titan-offline-spoof](../crates/titan-offline-spoof), `offline-hive` feature | Fixture / admin CI |
| GPU-PV 40 路 | R5 | [gpu_pv](../crates/titan-vmm/src/hyperv/gpu_pv.rs), host config templates | Provision dry-run |
| Capture + NVENC + WebRTC | R5 | `Capabilities::{streaming_nvenc, streaming_webrtc, streaming_precheck}` | Future milestone |
| WinDivert 转发 | R5 | `windivert` + kernel forward | Future milestone |

## M2 orchestration contract (default)

- **Transport**: framed binary in [wire.rs](../crates/titan-common/src/wire.rs) (`PROTOCOL_VERSION` in [lib.rs](../crates/titan-common/src/lib.rs)).
- **Spoof apply**: `ApplySpoofProfile` carries `vm_name`, `dry_run`, and optional `spoof` payload (`VmSpoofProfile`); host executes PowerShell steps with timeouts and optional audit log path.
- **Spoof step** (optional fine control): `ApplySpoofStep { vm_name, step_id, dry_run }` for single-step replay / debugging.
- **Capabilities**: built via [`Capabilities::from_host_runtime_probes`](../crates/titan-common/src/capabilities.rs) from [`host_runtime_probes::probe_host_runtime_blocking`](../crates/titan-host/src/host_runtime_probes.rs) at `serve` startup (`hyperv_spoof_host`, `kernel_driver_ipc`, vision flags — honest defaults until wired).
- **Guest agent binding**: `RegisterGuestAgent` from [titan-center](../crates/titan-center/src/app.rs) (after Connect) or any M2 client; optional UDP `DiscoveryBeacon` only advertises host control address for custom automation.
