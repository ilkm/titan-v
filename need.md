# 需求说明

> **仓库当前交付阶段（Phase 1）**：与 PR 验收默认对齐「中控↔宿主 **QUIC + mTLS** 控制面（替代旧 TCP 实现）+ **OpenVMM** 虚拟化能力集成（不再自研完整 Hyper-V 管理栈）+ 协作式 Guest Agent」及配套 Lua/配置能力。下文描述**长期产品形态**（高级虚拟机 fabric）与 **titan-host 元能力**意图 API；**哪些已在代码中闭环**以 **`need.md` 相关章节与当前 PR 说明**（Phase 1 Definition of Done / Phase 2+ 边界）为准，避免将路线图误读为已交付能力。
>
> 本文档描述的技术可用于合法自动化、安全研究与自有软件测试；对第三方软件与在线服务的滥用可能违反服务条款或法律。工程文档**不**保证反作弊或 EULA 合规；宿主/来宾安全启动与驱动策略见 `docs/openvmm-secure-boot-matrix.md`。

## 产品与角色

| 组件 | 职责 |
|------|------|
| **titan-center（中控端）** | 编排多台宿主机及其上的虚拟机：状态聚合、策略下发、脚本（Lua）与资源视图；长期目标含多路预览与流式交互。代码：`apps/titan-center`。 |
| **titan-host（被控端）** | 安装在每台宿主机上，对接具体虚拟化后端，向上暴露**元能力**（内存、伪装、输入、视觉、网络）；执行 provision、电源、每 VM Lua 运行时等。代码：`apps/titan-host`。 |
| **来宾 VM** | 承载业务负载的隔离环境；与宿主协作或通过底层 API 被宿主观测/操控（取决于阶段与能力位）。 |

## 规模用例（非架构前提）

在「约 200 台物理机、单宿主机约 40 VM、合计约 8000 窗口」一类规模下：中控做全局调度与代理池视图，宿主侧做差分盘批量克隆、多路 Lua、按 VM 的网络策略。具体数字随硬件与后端可调；需求以**元能力**与**控制面契约**为主。

## 宿主虚拟化后端

**产品路径**：宿主 OS **以 Windows 为主**（与 OpenVMM 支持矩阵一致），虚拟化后端 **[OpenVMM](https://openvmm.dev/)**（库或侧车进程 + 窄协议）；存储与镜像形态（母盘 / 差分盘 / IGVM 等）**以上游 OpenVMM 与集成设计为准**，由 `titan-host` 适配层编排 provision / 电源 / 设备能力。`titan-center`（`apps/titan-center`）可在其他桌面 OS 上运行以连接宿主。

中控与宿主之间的**默认控制面**为带版本号的二进制帧（`crates/titan-common` 中 `wire` / `PROTOCOL_VERSION`）；能力协商见 `Capabilities` 与宿主启动探测。

分层与演进说明见 `docs/host-windows-architecture.md`。

---

## titan-host 元能力（宿主侧抽象 API）

以下名称为**产品/契约层**意图；下列出 **Windows / OpenVMM** 主要底层落点。**并非**所有调用都会或应该以 QUIC `ControlRequest` 暴露（调试与 orchestrator 内部路径见 **`need.md` 与 PR 说明** 的对照约定）。

### 一、内存操控元能力（Memory Sovereignty）

在**不依赖来宾内核配合**的前提下，由宿主侧对 VM **物理地址空间**进行观测与修改（工程上即 hypervisor / WHV 视角的 guest physical memory）。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_read_raw(addr: u64, len: u32) -> Bytes` | 按 guest **物理**地址读取 | `WHvReadGuestPhysicalMemory` |
| `vm_write_raw(addr: u64, data: Bytes)` | 按 guest **物理**地址写入 | `WHvWriteGuestPhysicalMemory` |
| `vm_virt_to_phys(cr3: u64, virt_addr: u64) -> u64` | 软件页表遍历，虚拟地址 → guest 物理地址 | 不依赖 Windows 来宾用户态 API |
| `vm_scan_pattern(pattern: String) -> Vec<u64>` | 多线程等在物理内存范围内做特征扫描 | 利用多核扫较大 guest RAM |

**Phase 提示**：真 WinHv/WHV 无协作路径属 Phase 2+；Phase 1 协作式读内存等见 **`need.md` 与 PR 说明** 的 Phase 划分。`Capabilities::winhv_guest_memory` 等位须与探测一致。

### 二、硬件伪装元能力（Spoofing & Stealth）

使每个 VM 在观察者视角呈现为一致、可配置或随机化的「独立物理机」特征集。

| 意图 API | 能力说明 | Windows 轨（主要手段） |
|----------|----------|-------------------------|
| `vm_set_cpu_mask(feature_bits: u64)` | 影响 CPUID / 特性暴露（含弱化虚拟化痕迹等目标） | OpenVMM / 处理器策略 / 驱动组合，见 Phase 2+ |
| `vm_modify_hive(hive_path: Path, entries: Map)` | **离线**编辑挂载磁盘上的注册表 Hive（如磁盘序列号、显卡名称） | VHDX 挂载 + Hive 解析：`titan-offline-spoof`（`offline-hive` feature） |
| `vm_randomize_hwid()` | 一键生成逻辑自洽的硬件标识（如 MAC 与厂商前缀一致） | `VmSpoofProfile` / `mother_image` / 宿主侧自动化步骤等 |

方案 B（宿主 SB 与来宾 SB/vTPM 组合）的工程边界见 `docs/openvmm-secure-boot-matrix.md`。

### 三、合成输入元能力（Input Injection）

通过总线级路径注入**原始 HID 风格**输入，避免典型用户态模拟痕迹（产品目标；实现随阶段变化）。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_send_mouse_report(x: i16, y: i16, button: u8)` | 鼠标位移/按键类报告 | OpenVMM 集成输入通道 / 宿主输入注入路径（能力位以实际实现为准） |
| `vm_send_key_report(key_code: u16, state: bool)` | 键盘按下/抬起 | 同上 |

**Phase 提示**：宿主侧原始输入注入能力为后续阶段里程碑；`titan-driver` 与宿主服务通道设计以 OpenVMM 集成方案为准。Phase 1 协作式 Agent 通道不等价于本元能力闭环。当前收敛构建不定义独立“输入注入 capability 位”，待后续接线完成后再新增并对外声明。

### 四、视觉捕捉元能力（Visual Perception）

支持「无缝模式」、中控缩略预览与像素级自动化。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_get_frame_buffer() -> RawImage` | 获取当前帧原始像素 | 宿主采集路径（因 OpenVMM 与 OS 组合而异，如 `Windows.Graphics.Capture` 等） |
| `vm_image_find(template: Image) -> Option<(x, y)>` | 在像素流上做模板匹配，结果可反馈 Lua | 宿主 Rust 实现 |

**Phase 提示**：完整采集 + NVENC + WebRTC 为路线图里程碑；`streaming_precheck` 等能力位见 `capabilities.rs`。

### 五、网络隔离元能力（Network Isolation）

按 VM（或按网卡/队列）强制流量走指定出口，支撑「一窗口一出口」类拓扑。

| 意图 API | 能力说明 | Windows 轨（目标底层） |
|----------|----------|-------------------------|
| `vm_set_proxy(proxy_url: String)` | 将该 VM 相关流量导入代理隧道 | WinDivert 等内核态分流/转发（路线图）；当前收敛版仓库不含 `proxy_pool` / 宿主 WinDivert 转发栈 |

**Phase 提示**：代理 / WinDivert 真转发为路线图；`Capabilities::windivert_forward` 等位诚实反映宿主探测结果（当前收敛构建无 WinDivert 用户态栈）。

---

## 技术栈摘要

- **语言与运行时**：Rust；宿主侧每 VM **Lua** 有界执行（`titan-host::runtime`）。
- **控制面**：rkyv 帧、版本化协议；中控发起、宿主 `serve` 响应。
- **Windows 纵深**：OpenVMM 编排的 VM 存储与设备、可选 GPU / 直通类能力（以上游为准）与（后续）驱动 IPC；与上节五大元能力一一对应。
- **驱动**：Ring-0 组件为 Phase 2+ 独立交付物；与 `titan-host` 用户态服务通过约定 IPC 衔接。

## 部署与运行流程（摘要）

1. **母盘**：在参考 VM 内安装系统、依赖与负载，Sysprep 等封装后作为只读母盘。
2. **一键多开**：中控或宿主 CLI 触发 provision（差分盘、可选 GPU、伪装 profile、自动上电等）；具体编排由 OpenVMM 集成与宿主策略承载。
3. **运行期**：Lua +（Phase 1）Guest Agent 协作；后续阶段逐步替换为 WinHv / OpenVMM 深度能力 / 采集 / WinDivert 等真路径。

## 文档索引

| 文档 | 用途 |
|------|------|
| `need.md` 与当前 PR | Phase 1 DoD、Phase 2+ 边界与主题 → 代码对照（以文档与 PR 为准，无单独 `need_mapping` 模块） |
| `docs/requirements-traceability.md` | 元能力 / API → 实现轨 / 代码锚点 / 测试 |
| `docs/host-windows-architecture.md` | titan-host Windows 分层、**OpenVMM** 与适配层关系、Capabilities/Lua 约束 |
| `docs/openvmm-secure-boot-matrix.md` | **OpenVMM 上下文** 的宿主/来宾 SB 与驱动矩阵 |
