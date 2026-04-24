//! `need.md` 需求到代码落点的维护表，以及 **Phase 1** 正式交付范围。
//!
//! 完整愿景文本：仓库根目录 `need.md`（长期方案，非单阶段全部实现）。`need.md` **顶层按五大元能力**（内存、伪装、输入、视觉、网络）与**多宿主后端矩阵**组织；**下表按主题与阶段映射到 crate**，供实现与 PR 对照。
//!
//! # Phase 1（当前仓库默认交付目标）
//!
//! **一句话**：**中控↔宿主 TCP 控制面（rkyv 帧）+ Hyper-V（差分盘 / 电源 / 可选 GPU-PV / provision 后管线）+ 协作式 Guest Agent（TCP/JSON）**；不含内核驱动、真 VMBus HID、NVENC/WebRTC、WinDivert 真转发。
//!
//! ## 与 `need.md` 的关系
//!
//! - `need.md`：产品愿景、三后端矩阵与**五大元能力**意图 API（含 Ring-0、流、WinDivert 等路线图），**保持为需求源**。
//! - **实现与 PR 验收**：默认以本节 **Phase 1 Definition of Done** 为准；若工作属于 Phase 2+，须在 PR 中显式说明并更新本文件与相关 Capabilities，避免误导「已具备流/内核 / 全量元能力 API」。
//!
//! ## Phase 1 范围内（应保持、可测、有文档边界）
//!
//! - TCP 控制面：`crate::wire`、`titan-host::serve`、`titan-center`（Hello/Ping、ListVms、批量电源、`LoadScriptVm`、可选 `agent-bindings`、Capabilities 诚实位）。
//! - Hyper-V：`titan-vmm::hyperv`（差分盘 + Gen2、`vm_power`、可选 `gpu_pv`、provision 后 `Orchestrator::post_provision_after_create`）、`titan-host::config` / `titan-host::batch` 批处理 provision（无二进制 CLI）。
//! - 协作式 Agent：`hyperv::guest_agent`、`HypervHostRuntime`、`agent-bindings.toml`；**非** WinHv / paravisor。
//! - Lua：`titan-host::runtime` 有界队列、每 VM 串行、墙钟超时。
//! - 母盘 / 低风险（Phase 1.x）：`hyperv::mother_image`（`VmSpoofProfile` / `apply_host_spoof_profile`、PowerShell Job 超时、`probe_spoof_host_caps_blocking`）、`titan-host::batch::run_spoof`（Plan / Apply）；**仍不含**离线 Hive 实作（见 `titan-offline-spoof` 占位 crate）。
//! - 代理配置：`proxy_pool`、`windivert` TOML **仅校验与 schema**，不接内核转发。
//! - `titan-driver`：trait 边界 + Noop，文档区分 `GuestAgentChannel` 与 `VmbusHidChannel`。
//! - **示例 host 配置**（可复制后改路径）：`crates/titan-host/tests/fixtures/host.phase1.example.toml`。
//!
//! ## Phase 1 Definition of Done（DoD）
//!
//! 1. 控制面：`PROTOCOL_VERSION` 与 wire 编解码有回归测试；`serve` 多帧会话与背压行为可测。
//! 2. Hyper-V：非 Windows 明确拒绝；Windows 上 provision / 电源路径可审计（日志不含敏感脚本全文）。
//! 3. Guest Agent：协议与无绑定时的拒绝文案清晰；有 mock TCP 单测。
//! 4. Capabilities：与真实能力一致（含 `guest_agent`、`streaming_precheck`、`gpu_partition`、`hardware_spoof` 与 `hyperv_spoof_host` 子探测等），不宣称 Phase 2+ 能力。
//! 5. `cargo fmt` / `clippy -D warnings` / `cargo test --workspace` 通过；**Mac** 上 ListVms/电源仍为占位；**Linux** 上可选 **`virsh`** 列表与批量电源（`titan-vmm::platform_vm`、`kvm::virsh_shell`），无 `virsh` 时行为与旧版一致（空列表 / 明确失败），**不**宣称 libvirt/QEMU 全栈或 guest 物理内存。
//!
//! ## Phase 2+（显式不在 Phase 1 DoD）— 按五大元能力标注
//!
//! - **Memory（内存操控）**：Ring-0 或 WHV 等路径上的真 **guest physical** 读写、页表遍历、无协作扫描；用户态仅通过 **IPC 探测** 声明驱动能力（`Capabilities::winhv_guest_memory` 等）。宿主/来宾策略矩阵见 `docs/hyperv-secure-boot-matrix.md`。
//! - **Spoofing（硬件伪装）**：内核态深度伪装与 **离线 Hive** 实管（`titan-offline-spoof` / `offline-hive`）；Phase 1.x 以 PowerShell + `VmSpoofProfile` / `mother_image` 为主。
//! - **Input（合成输入）**：真 **VMBus HID** 注入（`Capabilities::vmbus_hid`）；非 Guest Agent 键盘鼠标捷径。
//! - **Visual（视觉捕捉）**：`Windows.Graphics.Capture` + NVENC + WebRTC 全链路；Phase 1 仅有 `streaming_precheck` 等诚实位。
//! - **Network（网络隔离）**：WinDivert **内核态**转发与大规模 SOCKS 性能验证（`Capabilities::windivert_forward`）；Phase 1 为配置 schema / 校验。
//! - **多后端**：macOS（`titan-vmm::hvf`）功能性后端仍为占位。Linux：**仅** libvirt **`virsh` shell** 路径上的 ListVms / 批量电源（见 `platform_vm`）；`ReadMemory` / 全设备模型等仍为 Phase 3+。
//!
//! ---
//!
//! ## 对照摘要（元能力 / need 主题 → 代码落点）
//!
//! **实现 PR 默认以 Phase 1 DoD 为准；下表含 Phase 2/3 落点供路线图对照。**
//!
//! | 元能力 / 主题 | 主要落点 |
//! |---------------|----------|
//! | Memory Sovereignty / WinHv | `Capabilities::winhv_guest_memory`、`titan-driver`、宿主 `host_runtime_probes`；Phase 2+ |
//! | Spoofing & Stealth / 母盘与 profile | `VmSpoofProfile`, `titan-vmm::hyperv::mother_image`, `titan-host::batch` / `VmProvisionPlan.spoof`, `HypervSpoofHostCaps`, `titan-offline-spoof`（Phase 2B） |
//! | Input Injection / VMBus HID | `Capabilities::{vmbus_input, vmbus_hid}`, `titan-driver`；Phase 2+ |
//! | Visual Perception / 流 | `Capabilities::{streaming_precheck, streaming_nvenc, streaming_webrtc}`；Phase 2+ / 路线图 |
//! | Network Isolation / 代理 | `crate::proxy_pool`, `titan-host::windivert`, `Capabilities::windivert_forward`；Phase 1 schema-only |
//! | 中控调度 / wire | `crate::wire`, `titan-host::serve`, `titan-center` |
//! | Hyper-V 差分 / 电源 | `titan-vmm::hyperv`, `titan-host::config` |
//! | GPU-PV | `titan-vmm::hyperv::gpu_pv` |
//! | 窗外读内存 / 注入（无驱动，协作式） | `titan-vmm::hyperv::guest_agent`, `HypervHostRuntime`（Phase 2A：`identity_echo` 协议占位） |
//! | Lua 多路 | `titan-host::runtime` |
//! | 驱动 / 真 VMBus | `titan-driver`（内核路径与 guest agent 分离文档） |
//! | KVM / Apple 后端 | `titan-vmm::platform_vm`（ListVms/电源分发）、`titan-vmm::kvm::virsh_shell`（Linux `virsh`）、`titan-vmm::kvm`、`titan-vmm::hvf`（其余能力仍占位） |
//! | Guest 读内存 / 鼠标（调试） | **未走控制面 TCP 帧**：由 `HypervHostRuntime` + orchestrator 内部调用；避免额外 `ControlRequest` 变体与版本矩阵 |
//! | 一键 provision / 上电链 | `titan-host::batch::run_provision` + `Orchestrator::post_provision_after_create`（hardware→GPU?→Start?→stream precheck）；`VmProvisionPlan` 可选 `gpu_partition_instance_path`、`auto_start_after_provision` |
//! | Phase 2：中控看板数据 | `titan-center` 可选定时 `ListVms`、slot 行绑定 `vm_inventory` + 选中 host 前缀 |
//! | Phase 3：流 / WinDivert / 内核 / 全量元能力 | **未在本仓库闭环**：Capture+NVENC+WebRTC；WinDivert 内核转发；Ring-0 驱动；KVM/Mac 实作 — 仅能力位与本文诚实标注 |
