# Requirements traceability (元能力 → 交付)

Rows tie [need.md](../need.md) **五大元能力**与意图 API 到实现轨、代码锚点与验证方式。意图 API 名为契约层别名；**未**承诺均已暴露为 TCP `ControlRequest`（内部/orchestrator 路径见 [`need_mapping.rs`](../crates/titan-common/src/need_mapping.rs)）。

宿主生产路径为 **Windows + Hyper-V**；分层与能力边界见 [host-windows-architecture.md](host-windows-architecture.md)。

| 元能力 | 意图 API（宿主侧抽象） | Windows / Hyper-V 轨 | 代码 / 文档锚点 | 测试 / E2E |
|--------|-------------------------|----------------------|-----------------|------------|
| Memory Sovereignty | `vm_read_raw` | Guest 物理内存读（`WHvReadGuestPhysicalMemory` 等） | `Capabilities::winhv_guest_memory`, `titan-driver`, `titan-host` runtime / probes | Phase 2+ 契约测试；Phase 1 不宣称 |
| Memory Sovereignty | `vm_write_raw` | Guest 物理内存写（`WHvWriteGuestPhysicalMemory`） | 同上 | 同上 |
| Memory Sovereignty | `vm_virt_to_phys` | 软件页表遍历 | 规划落 `titan-vmm` + host；当前无公开帧 | 单元 / 仿真 fixture（未来） |
| Memory Sovereignty | `vm_scan_pattern` | 多线程扫 guest 物理范围 | 同上 | 性能与正确性基准（未来） |
| Spoofing & Stealth | `vm_set_cpu_mask` | CPUID / 处理器暴露策略 + 未来驱动 | `VmSpoofProfile`, `mother_image`, `HypervSpoofHostCaps` | Windows IT / 单元 |
| Spoofing & Stealth | `vm_modify_hive` | 离线挂载 VHDX 编辑 Hive | [titan-offline-spoof](../crates/titan-offline-spoof), `offline-hive` feature | Fixture / admin CI（Phase 2B+） |
| Spoofing & Stealth | `vm_randomize_hwid` | `VmSpoofProfile` + 宿主自动化步骤 | `mother_image`, `ControlRequest::ApplySpoofProfile` | serve 集成测试 + 手工 VM 模板 |
| Spoofing & Stealth | 方案 B（宿主 SB / 驱动 / 来宾 vTPM） | 矩阵与里程碑 | [hyperv-secure-boot-matrix.md](hyperv-secure-boot-matrix.md), `VmIdentityProfile::host_kernel_driver_expected` | Manual / driver repo CI |
| Input Injection | `vm_send_mouse_report` | VMBus HID 原始报告 | `Capabilities::vmbus_hid`, driver IPC（`titan-driver`） | Driver 集成（Phase 2+） |
| Input Injection | `vm_send_key_report` | 同上 | `Capabilities::vmbus_input`（总输入能力伞） | 同上 |
| Visual Perception | `vm_get_frame_buffer` | `Windows.Graphics.Capture` / `vmwp` | `streaming_precheck`, 未来 capture 模块 | Future milestone |
| Visual Perception | `vm_image_find` | 宿主 Rust 像素/模板算法 → Lua | 同上 + `titan-host::runtime` | Future milestone |
| Network Isolation | `vm_set_proxy` | WinDivert 劫持/转发至代理隧道 | `proxy_pool`, `titan-host::windivert`, `Capabilities::windivert_forward` | Phase 1：schema 校验；转发为 Future |
| 编排（伪装流水线） | （控制面操作，非单 API） | `ApplySpoofProfile` / `ApplySpoofStep`；非 Windows：501 | [`crate::wire`](../crates/titan-common/src/wire/mod.rs) | serve 集成测试 |
| 编排（ListVms / 批量电源） | `ListVms`、`StartVmGroup` / `StopVmGroup` | Hyper-V / PowerShell；[`platform_vm`](../crates/titan-vmm/src/platform_vm.rs) 路由 | [`platform_vm`](../crates/titan-vmm/src/platform_vm.rs), [`dispatch`](../crates/titan-host/src/serve/dispatch.rs) | Windows 集成；非 Windows 构建为 stub |
| 规模与算力 | GPU-PV 多实例 | `gpu_pv`、宿主模板 | [gpu_pv](../crates/titan-vmm/src/hyperv/gpu_pv.rs) | Provision dry-run |

## Control-plane orchestration（默认）

中控通过帧协议编排宿主；**元能力**中仅部分（如伪装 profile 应用）已映射到稳定 `ControlRequest`。

- **Transport**: framed binary in [`wire` module](../crates/titan-common/src/wire/mod.rs) (`PROTOCOL_VERSION` in [lib.rs](../crates/titan-common/src/lib.rs)).
- **Spoof apply**: `ApplySpoofProfile` carries `vm_name`, `dry_run`, and optional `spoof` payload (`VmSpoofProfile`); host executes PowerShell steps with timeouts and optional audit log path.
- **Spoof step** (optional fine control): `ApplySpoofStep { vm_name, step_id, dry_run }` for single-step replay / debugging.
- **Capabilities**: built via [`Capabilities::from_host_runtime_probes`](../crates/titan-common/src/capabilities.rs) from [`host_runtime_probes::probe_host_runtime_blocking`](../crates/titan-host/src/host_runtime_probes.rs) at `serve` startup (`hyperv_spoof_host`, `kernel_driver_ipc`, vision flags — honest defaults until wired).
- **Guest agent binding**: configure `agent-bindings.toml` on the host (VM name → guest agent TCP); optional UDP `DiscoveryBeacon` only advertises the host control-plane TCP listen address for custom automation.
