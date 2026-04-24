# Requirements traceability (元能力 → 交付)

Rows tie [need.md](../need.md) **五大元能力**与意图 API 到实现轨、代码锚点与验证方式。意图 API 名为契约层别名；**未**承诺均已暴露为 TCP `ControlRequest`（内部/orchestrator 路径见 [`need_mapping.rs`](../crates/titan-common/src/need_mapping.rs)）。

下表「Linux / Mac 轨」列为占位级说明；**跨宿主 OS 分层、后端扩写、WHP 与 Hyper-V 管理轨关系、Lua/Capabilities 约束与里程碑**见 [host-cross-platform-architecture.md](host-cross-platform-architecture.md)。

| 元能力 | 意图 API（宿主侧抽象） | Windows / Hyper-V 轨 | Linux / Mac 轨（占位） | 代码 / 文档锚点 | 测试 / E2E |
|--------|-------------------------|----------------------|-------------------------|-----------------|------------|
| Memory Sovereignty | `vm_read_raw` | Guest 物理内存读（`WHvReadGuestPhysicalMemory` 等） | KVM：等价 hypervisor 内存 API（路线图） | `Capabilities::winhv_guest_memory`, `titan-driver`, `titan-host` runtime / probes | Phase 2+ 契约测试；Phase 1 不宣称 |
| Memory Sovereignty | `vm_write_raw` | Guest 物理内存写（`WHvWriteGuestPhysicalMemory`） | Mac：`hv`/`Hypervisor` 框架 guest RAM（路线图） | 同上 | 同上 |
| Memory Sovereignty | `vm_virt_to_phys` | 软件页表遍历 | 各后端页表格式适配（路线图） | 规划落 `titan-vmm` + host；当前无公开帧 | 单元 / 仿真 fixture（未来） |
| Memory Sovereignty | `vm_scan_pattern` | 多线程扫 guest 物理范围 | 同上 | 同上 | 性能与正确性基准（未来） |
| Spoofing & Stealth | `vm_set_cpu_mask` | CPUID / 处理器暴露策略 + 未来驱动 | 后端相关 CPU 模型与特性位（路线图） | `VmSpoofProfile`, `mother_image`, `HypervSpoofHostCaps` | Windows IT / 单元 |
| Spoofing & Stealth | `vm_modify_hive` | 离线挂载 VHDX 编辑 Hive | 非 Windows：磁盘上配置格式另行定义 | [titan-offline-spoof](../crates/titan-offline-spoof), `offline-hive` feature | Fixture / admin CI（Phase 2B+） |
| Spoofing & Stealth | `vm_randomize_hwid` | `VmSpoofProfile` + 宿主自动化步骤 | 占位 | `mother_image`, `ControlRequest::ApplySpoofProfile` | serve 集成测试 + 手工 VM 模板 |
| Spoofing & Stealth | 方案 B（宿主 SB / 驱动 / 来宾 vTPM） | 矩阵与里程碑 | N/A（矩阵按轨拆分） | [hyperv-secure-boot-matrix.md](hyperv-secure-boot-matrix.md), `VmIdentityProfile::host_kernel_driver_expected` | Manual / driver repo CI |
| Input Injection | `vm_send_mouse_report` | VMBus HID 原始报告 | 占位 | `Capabilities::vmbus_hid`, driver IPC（`titan-driver`） | Driver 集成（Phase 2+） |
| Input Injection | `vm_send_key_report` | 同上 | 占位 | `Capabilities::vmbus_input`（总输入能力伞） | 同上 |
| Visual Perception | `vm_get_frame_buffer` | `Windows.Graphics.Capture` / `vmwp` | 占位 | `streaming_precheck`, 未来 capture 模块 | Future milestone |
| Visual Perception | `vm_image_find` | 宿主 Rust 像素/模板算法 → Lua | 占位 | 同上 + `titan-host::runtime` | Future milestone |
| Network Isolation | `vm_set_proxy` | WinDivert 劫持/转发至代理隧道 | 占位 | `proxy_pool`, `titan-host::windivert`, `Capabilities::windivert_forward` | Phase 1：schema 校验；转发为 Future |
| 编排（伪装流水线） | （控制面操作，非单 API） | `ApplySpoofProfile` / `ApplySpoofStep` | 非 Windows：501；Linux 无额外伪装轨 | [`crate::wire`](../crates/titan-common/src/wire/mod.rs) | serve 集成测试 |
| 编排（ListVms / 批量电源） | `ListVms`、`StartVmGroup` / `StopVmGroup` | Hyper-V / PowerShell | Linux：`virsh`（`platform_vm`、`kvm::virsh_shell`）；`Capabilities.linux_virsh_inventory` | [`platform_vm`](../crates/titan-vmm/src/platform_vm.rs), [`dispatch`](../crates/titan-host/src/serve/dispatch.rs) | 全平台单元 / 集成；Linux 需本机 `virsh` 才非空 |
| 规模与算力 | GPU-PV 多实例 | `gpu_pv`、宿主模板 | 占位 | [gpu_pv](../crates/titan-vmm/src/hyperv/gpu_pv.rs) | Provision dry-run |

## Control-plane orchestration（默认）

中控通过帧协议编排宿主；**元能力**中仅部分（如伪装 profile 应用）已映射到稳定 `ControlRequest`。

- **Transport**: framed binary in [`wire` module](../crates/titan-common/src/wire/mod.rs) (`PROTOCOL_VERSION` in [lib.rs](../crates/titan-common/src/lib.rs)).
- **Spoof apply**: `ApplySpoofProfile` carries `vm_name`, `dry_run`, and optional `spoof` payload (`VmSpoofProfile`); host executes PowerShell steps with timeouts and optional audit log path.
- **Spoof step** (optional fine control): `ApplySpoofStep { vm_name, step_id, dry_run }` for single-step replay / debugging.
- **Capabilities**: built via [`Capabilities::from_host_runtime_probes`](../crates/titan-common/src/capabilities.rs) from [`host_runtime_probes::probe_host_runtime_blocking`](../crates/titan-host/src/host_runtime_probes.rs) at `serve` startup (`hyperv_spoof_host`, `kernel_driver_ipc`, vision flags — honest defaults until wired).
- **Guest agent binding**: configure `agent-bindings.toml` on the host (VM name → guest agent TCP); optional UDP `DiscoveryBeacon` only advertises the host control-plane TCP listen address for custom automation.
