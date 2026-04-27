# 需求说明

> **仓库当前交付阶段（Phase 1）**：与 PR 验收默认对齐「中控↔宿主 TCP 控制面 + Windows Hyper-V 自动化 + 协作式 Guest Agent」及配套 Lua/配置能力。下文描述**长期产品形态**（高级虚拟机 fabric）与 **titan-host 元能力**意图 API；**哪些已在代码中闭环**以 `crates/titan-common/src/need_mapping.rs`（Phase 1 Definition of Done / Phase 2+ 边界）为准，避免将路线图误读为已交付能力。
>
> 本文档描述的技术可用于合法自动化、安全研究与自有软件测试；对第三方软件与在线服务的滥用可能违反服务条款或法律。工程文档**不**保证反作弊或 EULA 合规；宿主/来宾安全启动与驱动策略见 `docs/hyperv-secure-boot-matrix.md`。

## 产品与角色

| 组件 | 职责 |
|------|------|
| **titan-center（中控端）** | 编排多台宿主机及其上的虚拟机：状态聚合、策略下发、脚本（Lua）与资源视图；长期目标含多路预览与流式交互。代码：`crates/titan-center`。 |
| **titan-host（被控端）** | 安装在每台宿主机上，对接具体虚拟化后端，向上暴露**元能力**（内存、伪装、输入、视觉、网络）；执行 provision、电源、每 VM Lua 运行时等。代码：`crates/titan-host`。 |
| **来宾 VM** | 承载业务负载的隔离环境；与宿主协作或通过底层 API 被宿主观测/操控（取决于阶段与能力位）。 |

## 规模用例（非架构前提）

在「约 200 台物理机、单宿主机约 40 VM、合计约 8000 窗口」一类规模下：中控做全局调度与代理池视图，宿主侧做差分盘批量克隆、多路 Lua、按 VM 的网络策略。具体数字随硬件与后端可调；需求以**元能力**与**控制面契约**为主。

## 宿主虚拟化后端

**产品路径**：宿主 OS **仅 Windows**，虚拟化后端 **Hyper-V**，存储目标为母盘（只读）+ 差分 VHDX；实作轨为 `crates/titan-vmm::hyperv`、宿主 provision / 电源 / GPU-PV 可选路径。`titan-center` 可在其他桌面 OS 上运行以连接 Windows 宿主。

中控与宿主之间的**默认控制面**为带版本号的二进制帧（`crates/titan-common` 中 `wire` / `PROTOCOL_VERSION`）；能力协商见 `Capabilities` 与宿主启动探测。

分层与演进说明见 `docs/host-windows-architecture.md`。

---

## titan-host 元能力（宿主侧抽象 API）

以下名称为**产品/契约层**意图；下列出 **Windows / Hyper-V** 主要底层落点。**并非**所有调用都会或应该以 TCP `ControlRequest` 暴露（调试与 orchestrator 内部路径见 `need_mapping.rs` 对照表）。

### 一、内存操控元能力（Memory Sovereignty）

在**不依赖来宾内核配合**的前提下，由宿主侧对 VM **物理地址空间**进行观测与修改（工程上即 hypervisor / WHV 视角的 guest physical memory）。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_read_raw(addr: u64, len: u32) -> Bytes` | 按 guest **物理**地址读取 | `WHvReadGuestPhysicalMemory` |
| `vm_write_raw(addr: u64, data: Bytes)` | 按 guest **物理**地址写入 | `WHvWriteGuestPhysicalMemory` |
| `vm_virt_to_phys(cr3: u64, virt_addr: u64) -> u64` | 软件页表遍历，虚拟地址 → guest 物理地址 | 不依赖 Windows 来宾用户态 API |
| `vm_scan_pattern(pattern: String) -> Vec<u64>` | 多线程等在物理内存范围内做特征扫描 | 利用多核扫较大 guest RAM |

**Phase 提示**：真 WinHv/WHV 无协作路径属 Phase 2+；Phase 1 协作式读内存等见 `need_mapping.rs`。`Capabilities::winhv_guest_memory` 等位须与探测一致。

### 二、硬件伪装元能力（Spoofing & Stealth）

使每个 VM 在观察者视角呈现为一致、可配置或随机化的「独立物理机」特征集。

| 意图 API | 能力说明 | Windows 轨（主要手段） |
|----------|----------|-------------------------|
| `vm_set_cpu_mask(feature_bits: u64)` | 影响 CPUID / 特性暴露（含隐藏 Hyper-V 相关标志等目标） | 处理器策略、驱动/虚拟化栈等组合，见 Phase 2+ |
| `vm_modify_hive(hive_path: Path, entries: Map)` | **离线**编辑挂载磁盘上的注册表 Hive（如磁盘序列号、显卡名称） | VHDX 挂载 + Hive 解析：`titan-offline-spoof`（`offline-hive` feature） |
| `vm_randomize_hwid()` | 一键生成逻辑自洽的硬件标识（如 MAC 与厂商前缀一致） | `VmSpoofProfile` / `mother_image` / PowerShell 宿主侧步骤等 |

方案 B（宿主 SB 与来宾 SB/vTPM 组合）的工程边界见 `docs/hyperv-secure-boot-matrix.md`。

### 三、合成输入元能力（Input Injection）

通过总线级路径注入**原始 HID 风格**输入，避免典型用户态模拟痕迹（产品目标；实现随阶段变化）。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_send_mouse_report(x: i16, y: i16, button: u8)` | 鼠标位移/按键类报告 | VMBus 合成 HID 路径；`Capabilities::vmbus_hid` |
| `vm_send_key_report(key_code: u16, state: bool)` | 键盘按下/抬起 | 同上 |

**Phase 提示**：真 VMBus HID 注入为 Phase 2+；`titan-driver` 区分 `GuestAgentChannel` 与 `VmbusHidChannel`。Phase 1 协作式 Agent 通道不等价于本元能力闭环。

### 四、视觉捕捉元能力（Visual Perception）

支持「无缝模式」、中控缩略预览与像素级自动化。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_get_frame_buffer() -> RawImage` | 获取当前帧原始像素 | `Windows.Graphics.Capture` 等监听 `vmwp.exe` 关联表面 |
| `vm_image_find(template: Image) -> Option<(x, y)>` | 在像素流上做模板匹配，结果可反馈 Lua | 宿主 Rust 实现 |

**Phase 提示**：完整采集 + NVENC + WebRTC 为路线图里程碑；`streaming_precheck` 等能力位见 `capabilities.rs`。

### 五、网络隔离元能力（Network Isolation）

按 VM（或按网卡/队列）强制流量走指定出口，支撑「一窗口一出口」类拓扑。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_set_proxy(proxy_url: String)` | 将该 VM 相关流量导入代理隧道 | WinDivert 等内核态分流/转发与配置 schema：`proxy_pool`、`windivert` |

**Phase 提示**：Phase 1 对代理 / WinDivert 配置多为 **TOML 校验与 schema**，不接内核转发；`Capabilities::windivert_forward` 等位诚实反映探测结果。

---

## 技术栈摘要

- **语言与运行时**：Rust；宿主侧每 VM **Lua** 有界执行（`titan-host::runtime`）。
- **控制面**：rkyv 帧、版本化协议；中控发起、宿主 `serve` 响应。
- **Windows 纵深**：Hyper-V 差分盘、可选 GPU-PV、PowerShell 自动化与（后续）驱动 IPC；与上节五大元能力一一对应。
- **驱动**：Ring-0 组件为 Phase 2+ 独立交付物；与 `titan-host` 用户态服务通过约定 IPC 衔接。

## 部署与运行流程（摘要）

1. **母盘**：在参考 VM 内安装系统、依赖与负载，Sysprep 等封装后作为只读母盘。
2. **一键多开**：中控或宿主 CLI 触发 provision：差分盘、（可选）GPU 分区、伪装 profile、自动上电等（见 `VmProvisionPlan`、`Orchestrator::post_provision_after_create`）。
3. **运行期**：Lua +（Phase 1）Guest Agent 协作；后续阶段逐步替换为 WinHv / VMBus / 采集 / WinDivert 等真路径。

## 文档索引

| 文档 | 用途 |
|------|------|
| `crates/titan-common/src/need_mapping.rs` | Phase 1 DoD、Phase 2+ 列表、主题 → crate 对照 |
| `docs/requirements-traceability.md` | 元能力 / API → 实现轨 / 代码锚点 / 测试 |
| `docs/host-windows-architecture.md` | titan-host Windows 分层、Hyper-V 与 WHP 关系、Capabilities/Lua 约束 |
| `docs/hyperv-secure-boot-matrix.md` | **仅 Windows / Hyper-V 轨** 的宿主/来宾 SB 与驱动矩阵 |
