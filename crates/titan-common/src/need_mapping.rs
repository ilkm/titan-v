//! `need.md` 需求到代码落点的维护表，以及 **Phase 1** 正式交付范围。
//!
//! 完整愿景文本：仓库根目录 `need.md`（长期方案，非单阶段全部实现）。`need.md` **顶层按五大元能力**（内存、伪装、输入、视觉、网络）与 **Windows / Hyper-V 宿主**组织；**下表按主题与阶段映射到 crate**，供实现与 PR 对照。
//!
//! # Phase 1（当前仓库默认交付目标）
//!
//! **一句话（当前快照）**：**中控↔宿主 TCP/QUIC 控制面（`titan-common::wire` 帧）** 在 `titan-host::serve` 实现；**Hyper-V / Lua / 离线 spoof / driver 独立 crate 已从本工作区移除**，相关 `ControlRequest` 分支返回 **空列表 / 501** 等降级语义。不含内核驱动、真 VMBus HID、NVENC/WebRTC、WinDivert 真转发。
//!
//! ## 与 `need.md` 的关系
//!
//! - `need.md`：产品愿景、**五大元能力**意图 API（含 Ring-0、流、WinDivert 等路线图），**保持为需求源**。
//! - **实现与 PR 验收**：默认以本节 **Phase 1 Definition of Done** 为准；若工作属于 Phase 2+，须在 PR 中显式说明并更新本文件与相关 Capabilities，避免误导「已具备流/内核 / 全量元能力 API」。
//!
//! ## Phase 1 范围内（应保持、可测、有文档边界）
//!
//! - TCP 控制面：`crate::wire`、`titan-host::serve`、`titan-center`（Hello/Ping、UI 下发、`HostDesktopSnapshot` 等；ListVms 空、批量电源与脚本/spoof 返回 **501**）。
//! - Hyper-V / 母盘 / Lua / driver：**原 `titan-vmm` / `titan-scripts` / `titan-driver` / `titan-offline-spoof` crate 已删除**；路线图与 `need.md` 仍以 Phase 描述为准，验收须对照 PR 是否恢复实现。
//! - `titan-host::config`：TOML schema 可保留供外部工具；宿主二进制不再内置批处理 provision。
//! - 代理配置：`proxy_pool`、`windivert` TOML **仅校验与 schema**，不接内核转发。
//! - **示例 host 配置**（可复制后改路径）：`apps/titan-host/tests/fixtures/host.phase1.example.toml`。
//!
//! ## Phase 1 Definition of Done（DoD）
//!
//! 1. 控制面：`PROTOCOL_VERSION` 与 wire 编解码有回归测试；`serve` 多帧会话与背压行为可测。
//! 2. Hyper-V：非 Windows 明确拒绝；Windows 上 provision / 电源路径可审计（日志不含敏感脚本全文）。
//! 3. Guest Agent：协议与无绑定时的拒绝文案清晰；有 mock TCP 单测。
//! 4. Capabilities：与真实能力一致（含 `guest_agent`、`streaming_precheck`、`gpu_partition`、`hardware_spoof` 与 `hyperv_spoof_host` 子探测等），不宣称 Phase 2+ 能力。
//! 5. `cargo fmt` / `clippy -D warnings` / `cargo test --workspace` 通过；Capabilities 与 **当前**宿主实现一致（无 Hyper-V crate 时不宣称 `hyperv` 等）。
//!
//! ## Phase 2+（显式不在 Phase 1 DoD）— 按五大元能力标注
//!
//! - **Memory（内存操控）**：Ring-0 或 WHV 等路径上的真 **guest physical** 读写、页表遍历、无协作扫描；用户态仅通过 **IPC 探测** 声明驱动能力（`Capabilities::winhv_guest_memory` 等）。宿主/来宾策略矩阵见 `docs/hyperv-secure-boot-matrix.md`。
//! - **Spoofing（硬件伪装）**：内核态深度伪装与 **离线 Hive** 实管（原独立 crate，已移除）；Phase 1.x 以 PowerShell + `VmSpoofProfile` / `mother_image` 为主（路线图）。
//! - **Input（合成输入）**：真 **VMBus HID** 注入（`Capabilities::vmbus_hid`）；非 Guest Agent 键盘鼠标捷径。
//! - **Visual（视觉捕捉）**：`Windows.Graphics.Capture` + NVENC + WebRTC 全链路；Phase 1 仅有 `streaming_precheck` 等诚实位。
//! - **Network（网络隔离）**：WinDivert **内核态**转发与大规模 SOCKS 性能验证（`Capabilities::windivert_forward`）；Phase 1 为配置 schema / 校验。
//!
//! ---
//!
//! ## 对照摘要（元能力 / need 主题 → 代码落点）
//!
//! **实现 PR 默认以 Phase 1 DoD 为准；下表含 Phase 2/3 落点供路线图对照。**
//!
//! | 元能力 / 主题 | 主要落点 |
//! |---------------|----------|
//! | Memory Sovereignty / WinHv | `Capabilities::winhv_guest_memory`、宿主 `host_runtime_probes`；Phase 2+ |
//! | Spoofing & Stealth / 母盘与 profile | `VmSpoofProfile`, `HypervSpoofHostCaps`（原 `mother_image` / 离线 crate 已移除；Phase 2B+） |
//! | Input Injection / VMBus HID | `Capabilities::{vmbus_input, vmbus_hid}`；Phase 2+ |
//! | Visual Perception / 流 | `Capabilities::{streaming_precheck, streaming_nvenc, streaming_webrtc}`；Phase 2+ / 路线图 |
//! | Network Isolation / 代理 | `crate::proxy_pool`, `titan-host::windivert`, `Capabilities::windivert_forward`；Phase 1 schema-only |
//! | 中控调度 / wire | `crate::wire`, `titan-host::serve`, `titan-center` |
//! | Hyper-V 差分 / 电源 | **已移除**（路线图：`need.md`）；`titan-host::config` schema 可保留 |
//! | GPU-PV | **已移除**（路线图） |
//! | 窗外读内存 / 注入（无驱动，协作式） | **已移除**（路线图） |
//! | Lua 多路 | **已移除**（`LoadScriptVm` → 501） |
//! | 驱动 / 真 VMBus | **已移除**（路线图） |
//! | ListVms / 域电源入口 | `titan-host::serve::dispatch`（空列表 / 501） |
//! | Guest 读内存 / 鼠标（调试） | **未实现**（原 `HypervHostRuntime` 路径已移除） |
//! | 一键 provision / 上电链 | **未实现**（原 `batch` / `Orchestrator` 已移除） |
//! | Phase 2：中控看板数据 | `titan-center` 可选定时 `ListVms`、slot 行绑定 `vm_inventory` + 选中 host 前缀 |
//! | Phase 3：流 / WinDivert / 内核 / 全量元能力 | **未在本仓库闭环**：Capture+NVENC+WebRTC；WinDivert 内核转发；Ring-0 驱动 — 仅能力位与本文诚实标注 |
