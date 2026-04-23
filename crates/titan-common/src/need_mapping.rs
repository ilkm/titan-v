//! `need.md` 需求到代码落点的维护表，以及 **Phase 1** 正式交付范围。
//!
//! 完整愿景文本：仓库根目录 `need.md`（长期方案，非单阶段全部实现）。
//!
//! # Phase 1（当前仓库默认交付目标）
//!
//! **一句话**：**M2 控制面 + Hyper-V（差分盘 / 电源 / 可选 GPU-PV / provision 后管线）+ 协作式 Guest Agent（TCP/JSON）**；不含内核驱动、真 VMBus HID、NVENC/WebRTC、WinDivert 真转发。
//!
//! ## 与 `need.md` 的关系
//!
//! - `need.md`：产品愿景与完整技术栈（含 Ring-0、流、WinDivert 等），**保持为需求源**。
//! - **实现与 PR 验收**：默认以本节 **Phase 1 Definition of Done** 为准；若工作属于 Phase 2+，须在 PR 中显式说明并更新本文件与相关 Capabilities，避免误导「已具备流/内核」。
//!
//! ## Phase 1 范围内（应保持、可测、有文档边界）
//!
//! - M2 控制面：`crate::wire`、`titan-host::serve`、`titan-center`（Hello/Ping、ListVms、批量电源、`LoadScriptVm`、可选 `agent-bindings`、Capabilities 诚实位）。
//! - Hyper-V：`titan-vmm::hyperv`（差分盘 + Gen2、`vm_power`、可选 `gpu_pv`、provision 后 `Orchestrator::post_provision_after_create`）、`titan-host::config` / `main` provision。
//! - 协作式 Agent：`hyperv::guest_agent`、`HypervHostRuntime`、`agent-bindings.toml`；**非** WinHv / paravisor。
//! - Lua：`titan-host::runtime` 有界队列、每 VM 串行、墙钟超时。
//! - 母盘 / 低风险（Phase 1.x）：`hyperv::mother_image`（`VmSpoofProfile` / `apply_host_spoof_profile`、PowerShell Job 超时、`probe_spoof_host_caps_blocking`）、`spoof plan` + `spoof apply` CLI；**仍不含**离线 Hive 实作（见 `titan-offline-spoof` 占位 crate）。
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
//! 5. `cargo fmt` / `clippy -D warnings` / `cargo test --workspace` 通过；KVM/Mac **保持占位**。
//!
//! ## Phase 2+（显式不在 Phase 1 DoD）
//!
//! - Ring-0 内核驱动、真 VMBus HID、WinHv 无协作读内存（用户态仅通过 **IPC 探测** 声明驱动能力；矩阵见 `docs/hyperv-secure-boot-matrix.md`）。
//! - `Windows.Graphics.Capture` + NVENC + WebRTC 全链路。
//! - WinDivert 内核态转发与大规模 SOCKS 性能验证。
//! - KVM/Mac 功能性后端（当前刻意 `NotImplemented`）。
//!
//! ---
//!
//! ## 对照摘要（need 主题 → 代码落点）
//!
//! **实现 PR 默认以 Phase 1 DoD 为准；下表含 Phase 2/3 落点供路线图对照。**
//!
//! | need 主题 | 主要落点 |
//! |-----------|----------|
//! | 中控调度 / wire | `crate::wire`, `titan-host::serve`, `titan-center` |
//! | Hyper-V 差分 / 电源 | `titan-vmm::hyperv`, `titan-host::config` |
//! | GPU-PV | `titan-vmm::hyperv::gpu_pv` |
//! | 母盘 / 去特征 | `titan-vmm::hyperv::mother_image`, `titan-host` spoof CLI / `VmProvisionPlan.spoof`, `titan-offline-spoof`（Phase 2B 占位） |
//! | 窗外读内存 / 注入（无驱动） | `titan-vmm::hyperv::guest_agent`, `HypervHostRuntime`（Phase 2A：`identity_echo` 协议占位） |
//! | Lua 40 路 | `titan-host::runtime` |
//! | 代理 / WinDivert 设计 | `crate::proxy_pool`, `titan-host::windivert` |
//! | 驱动 / 真 VMBus | `titan-driver`（内核路径与 guest agent 分离文档） |
//! | KVM/Mac | `titan-vmm::kvm`, `titan-vmm::mac`（刻意占位） |
//! | Guest 读内存 / 鼠标（调试） | **未上 M2 wire**：由 `HypervHostRuntime` + orchestrator 内部调用；避免额外 `ControlRequest` 变体与版本矩阵 |
//! | 一键启动 L116-118 / 挂机链 L119-120 | `titan-host::main` `run_one_vm` + `Orchestrator::post_provision_after_create`（hardware→GPU?→Start?→stream precheck）；`VmProvisionPlan` 可选 `gpu_partition_instance_path`、`auto_start_after_provision` |
//! | Phase 2：中控看板数据 | `titan-center` 可选定时 `ListVms`、slot 行绑定 `vm_inventory` + 选中 host 前缀 |
//! | Phase 3：流 / WinDivert / 内核 | **未在本仓库闭环**：`Windows.Graphics.Capture`+NVENC+WebRTC；WinDivert 内核转发；Ring-0 驱动 — 仅能力位与本文诚实标注 |
