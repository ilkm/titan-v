# QUIC + mTLS 传输层迁移设计

> **Phase 声明**：本工作把 `need.md` 第 3 行的「中控↔宿主 TCP 控制面」**升级到 Phase 2+ 的 QUIC + mTLS 控制面**。合并前会同步更新
> [`need.md`](../need.md) Phase 1 段、[`requirements-traceability.md`](requirements-traceability.md) Transport 行、`titan_common::PROTOCOL_VERSION`，并对能力模型给出最终口径（当前不新增 `quic_mtls_transport` 字段，传输层默认固定为 QUIC + mTLS）。
>
> 本文是 review 文档；下一轮按本文落实代码，不再做架构再设计（避免再次反向）。

---

## 1. 决策摘要（已确认）

| 决策 | 选定 |
|---|---|
| 范围 | 提升至 Phase 2+；同 PR 内全量替换 TCP，迁移所有集成测试，清理 TCP 代码 |
| 通道 | 控制面 + telemetry 都走 QUIC；同一 connection 内多路复用 |
| 控制面 stream 模型 | **一条 bi-stream = 一次 RPC**，完成即关；天然规避队头阻塞 |
| Telemetry stream 模型 | **一条长 uni stream**（Host→Center）顺序写 `ControlPush` |
| 发现层 | **保留 UDP 广播 announce**（QUIC 不能广播）；announce 帧附带 SPKI fingerprint |
| TLS | mTLS（双方都验对方证书） |
| Cert 生成 | `rcgen` ed25519 自签，CN/SAN = `device_id`；首次启动写盘后持久化 |
| Center 信任 host | UDP announce 的 fingerprint 自动入信任表；手动加 host 时 TOFU 弹窗 |
| Host 信任 Center | Host 设置面板「开放配对 5 分钟」窗口期内首个连入的 Center 自动入信任；窗口关闭后只验信任表 |
| QUIC 库 | `quinn` 0.11（基于 `rustls` 0.23） |

---

## 2. 协议与端口

### 2.1 ALPN 与版本

```text
ALPN_CONTROL_V1   = "titan-control-v1"
ALPN_TELEMETRY_V1 = "titan-telemetry-v1"
PROTOCOL_VERSION  = 16   // 当前 15，本次 +1
```

- ALPN 与 `PROTOCOL_VERSION` **解耦但同步演进**：当 wire 类型在 rkyv 层有破坏性变更时，**同步**新增
  `titan-control-v2` 等并 bump `PROTOCOL_VERSION`。本次仅是传输替换，wire 类型基本不动，但仍 bump 一次以
  与旧 TCP 协议族明确区分。

### 2.2 帧格式

QUIC stream 内复用现有 **`titan_common::wire::codec` 的 rkyv 帧**（`MAGIC + version + len + payload`），
保留 `parse_header` / `decode_*_payload` 路径，**不重写 codec**。一条 RPC 在 bi-stream 上的形态：

```text
Center                                     Host
  ──── ControlRequestFrame ─────────►        // 一条 bi-stream，一次写
        (Center 关闭其 send 半流)
                                  ◄────── ControlHostFrame::Response ────
        (Host 写完关闭 send 半流)
  双方读到 Fin 即结束这条 stream
```

Telemetry uni stream 形态：

```text
Host: [TelemetryFrame {push}] [TelemetryFrame {push}] ...
                  ↓ Fin (Host 关闭 endpoint 时)
Center: 解析 + 分发到 NetUiMsg::HostTelemetry
```

### 2.3 端口

- **单 UDP 端口（沿用 7788）** 同时承载 control + telemetry：QUIC 的 ALPN 区分协议族。
- 删除 `CONTROL_PLANE_TELEMETRY_PORT_OFFSET` 及 `control_plane_telemetry_addr`（保留 `pub`
  re-export 的 `#[deprecated]` 别名指向新 fn 一个 release 周期，再删）。
- UDP 发现端口（7789 / 7791 / 7792）保持不变。

### 2.4 UDP `HostAnnounceBeacon`（重写）

```rust
pub const HOST_ANNOUNCE_SCHEMA_VERSION: u32 = 3;

pub struct HostAnnounceBeacon {
    pub kind: String,
    pub schema: u32,                    // 必须 == 3
    pub host_quic_addr: String,         // udp:port，QUIC endpoint
    pub label: String,
    pub device_id: String,              // 必填，非空
    pub host_spki_sha256_hex: String,   // 必填，64-char lowercase hex
}
```

新项目：**不维护 v1/v2 兼容路径**，schema 必须等于 3，所有字段必须非空，否则解析直接拒绝（`validate()` 返回 Err）。
旧字段名 `host_control_addr` 同步改名 `host_quic_addr`，避免误以为是 TCP。

---

## 3. 证书与信任

### 3.1 自签证书

- 算法：**ed25519**（rcgen 0.13 已稳定支持，最快握手；rustls 0.23 默认开启）。
- 文件：
  - Center：`~/Library/Application Support/titan-center/identity.{cert,key}`（macOS）/ `%LOCALAPPDATA%\titan-center\identity.*`（Win）/ XDG `data_local_dir` 下同名（Linux）
  - Host：`~/.../titan-host/identity.{cert,key}`
- 命名：CN = `titan-{role}-{device_id}`，SAN.DNS = 同 CN，SAN.IP = `0.0.0.0`（让对端不基于 SAN
  名称校验，只走 fingerprint）。
- 有效期：100 年（自签，靠 fingerprint 钉死，免去续签流程）。
- 模块：新 crate `crates/titan-quic`，文件 `cert_store.rs`：
  ```rust
  pub fn load_or_generate_identity(role: Role, device_id: &str) -> anyhow::Result<Identity>
  pub struct Identity { cert_der: Vec<u8>, key_der: Vec<u8> }
  pub fn spki_sha256_hex(cert_der: &[u8]) -> String
  ```

### 3.2 trust store

新表（Center 与 Host 各自独立）：

```sql
CREATE TABLE IF NOT EXISTS trusted_peers (
    peer_kind TEXT NOT NULL CHECK(peer_kind IN ('center','host')),
    peer_device_id TEXT NOT NULL,
    spki_sha256_hex TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    added_at_unix_ms INTEGER NOT NULL,
    PRIMARY KEY (peer_kind, peer_device_id)
);
```

- 落地：
  - Center: `apps/titan-center/src/app/trust_store.rs`，文件 `trusted_peers.sqlite`，env override
    `TITAN_CENTER_TRUST_DB_PATH`
  - Host: `apps/titan-host/src/trust_store.rs`，文件 `trusted_peers.sqlite`，env override
    `TITAN_HOST_TRUST_DB_PATH`
- API 完全对称：`list_all / upsert / remove / contains(peer_kind, fingerprint)`。

### 3.3 mTLS 校验路径

- **Center 作 client，Host 作 server**：
  - Center 出示 client cert；Host 校验 SPKI fingerprint 是否在 trust store；不在 → 检查 pairing
    window；窗口期内自动写入信任并放行；窗口关闭则握手失败（`ConnectionError::ConnectionClosed`）。
  - Host 出示 server cert；Center 校验 SPKI fingerprint：
    - UDP announce 已带 fingerprint → 已写入 trust → 直接放行
    - 手填添加（无 announce）→ 触发 TOFU 弹窗：显示 fingerprint，用户点 "信任并连接" 才入 trust。
- **rustls 自定义 verifier**：用 `rustls::client::danger::ServerCertVerifier` /
  `rustls::server::danger::ClientCertVerifier` 替换默认 PKI 校验，**只对比 fingerprint**。

### 3.4 Pairing window 状态机（Host 端）

```text
[Closed]  ──UI 点击「开放配对 5 分钟」──►  [Open(deadline)]
                                                  │
                          5 min 到 / UI 点击关闭   │
                                                  ▼
                                              [Closed]

[Open(deadline)]: 任何首次连入的 Center 客户端 cert
                  → 写入 trust store
                  → 写入后 deadline 立即缩短到 NOW（关闭窗口，防止多 Center 抢注）
```

- 状态存内存即可（重启即关），不持久化。
- 窗口期 deadline 暴露到 Host UI 状态条，倒计时显示。

---

## 4. 连接生命周期

### 4.1 单一 QUIC connection 复用

- Center 对每个已知 host **维持一条长连 QUIC connection**（替代当前 "命令短连接 + telemetry 长连接" 双通道）。
- 控制面 RPC = 在该 connection 上 `open_bi()` 一条 bi-stream，发完即关。
- Telemetry = Center 在 connection 建立后调一次 RPC `ControlRequest::SubscribeTelemetry`，Host 收到
  后 `open_uni()` 一条 uni stream 持续推；Center 持续读直到 stream Fin / connection 关闭。

### 4.2 重连与背压

- Center 端 reconnect 退避：`100ms → 200ms → 400ms → ... → 5s` 上限，连接成功复位。
- QUIC 层 keep-alive `Duration::from_secs(15)`；idle timeout `Duration::from_secs(60)`。
- Telemetry stream 写 backpressure：Host 通过 `quinn::SendStream::write_all` 自然反压，避免无界 buffer。

### 4.3 关停

- Host serve 关停（`watch::Sender<bool>` 切 true）→ `endpoint.close(...)`；所有 stream Fin；Center 读到
  `ConnectionError::LocallyClosed/ApplicationClosed` 走重连。

---

## 5. 文件清单

### 5.1 新增

| 路径 | 用途 |
|---|---|
| `crates/titan-quic/Cargo.toml`、`src/lib.rs` | 新 crate，统一封装 |
| `crates/titan-quic/src/identity.rs` | cert/key 生成 + 持久化 + fingerprint |
| `crates/titan-quic/src/endpoint.rs` | server / client `quinn::Endpoint` 工厂；自定义 verifier |
| `crates/titan-quic/src/alpn.rs` | ALPN 常量 |
| `crates/titan-quic/src/frame_io.rs` | `read_one_request / write_response / read_push_loop` 在 quinn stream 上的封装（复用 wire codec） |
| `apps/titan-center/src/app/trust_store.rs` | Center 信任表 + tests |
| `apps/titan-host/src/trust_store.rs` | Host 信任表 + tests |
| `apps/titan-host/src/host_app/pairing.rs` | pairing window 状态 + 倒计时 |
| `apps/titan-host/tests/quic_apply_vm_window_snapshot.rs` | 替代当前 `serve_apply_vm_window_snapshot.rs` |
| `apps/titan-host/tests/quic_hello.rs` 等 | 替代 `serve_hello / serve_ping / serve_multiframe / serve_apply_spoof_profile` |

### 5.2 修改

| 路径 | 改动 |
|---|---|
| `crates/titan-common/src/lib.rs` | `PROTOCOL_VERSION = 16`；移除 `CONTROL_PLANE_TELEMETRY_PORT_OFFSET` / `control_plane_telemetry_addr` re-export |
| `crates/titan-common/src/wire/mod.rs` | 同上：清理 telemetry 偏移端口常量；保留 codec |
| `crates/titan-common/src/discovery.rs` | `HOST_ANNOUNCE_SCHEMA_VERSION=3` + `host_spki_sha256_hex`；保留 schema v1/v2 解析（不入信任） |
| `apps/titan-host/Cargo.toml` | 添加 `titan-quic` 依赖；移除 `tcp_tune` 用不上的 socket2 项（如有） |
| `apps/titan-host/src/serve/run.rs` | 重写：`quinn::Endpoint::server` 替代 `tcp_listen_tokio`；删除 telemetry 单独 listener；accept 循环走 QUIC connection |
| `apps/titan-host/src/serve/io.rs` | 重写 `read_one_control_request`：从 quinn stream 而非 TcpStream |
| `apps/titan-host/src/serve/state.rs` | 加入 trust_store 引用 + pairing 状态指针 |
| `apps/titan-host/src/serve/dispatch.rs` | 加 `ControlRequest::SubscribeTelemetry` 分支（新 wire 项），转发到 connection 持有者 |
| `apps/titan-host/src/serve/run.rs` | 新建/管理 telemetry uni stream 写循环（每 connection 一条） |
| `apps/titan-host/src/serve/announce.rs` | beacon 包加 fingerprint |
| `apps/titan-host/src/host_app/shell/serve_bootstrap.rs` | 启动加载/生成 cert；构造 trust store |
| `apps/titan-host/src/host_app/ui/settings.rs`（或对应面板）| pairing 按钮 + 已信任 Center 列表 |
| `apps/titan-host/src/tcp_tune.rs` | 删除（不再用 TCP） |
| `apps/titan-center/Cargo.toml` | 添加 `titan-quic` 依赖 |
| `apps/titan-center/src/app/net/client.rs` | `exchange_one` 重写为 quinn bi-stream 一次 RPC |
| `apps/titan-center/src/app/net/connection_pool.rs`（新文件） | 每 host 一条长连 QUIC connection 的池子；重连退避；提供 `with_connection(addr, fn)` |
| `apps/titan-center/src/app/spawn/telemetry.rs`（或对应位置）| Telemetry reader：建 connection → SubscribeTelemetry → 持续读 uni stream |
| `apps/titan-center/src/app/center_shell/bootstrap.rs` | 启动加载/生成 cert；构造 trust store；用 announce 自动写入 fingerprint |
| `apps/titan-center/src/app/center_shell/net_lan.rs` | host announce 处理：自动 trust upsert |
| `apps/titan-center/src/app/ui/devices/add_host_dialog.rs`（或现有手动加 host 入口）| TOFU 弹窗：显示 host fingerprint，用户确认后入 trust |
| `apps/titan-center/src/app/tcp_tune.rs` | 删除 |

### 5.3 删除

- `apps/titan-host/src/tcp_tune.rs`、`apps/titan-center/src/app/tcp_tune.rs`（QUIC 不需要 socket2 nodelay 调优）
- 旧 TCP 集成测试（按 5.1 列出的方式整套替换）
- `wire::CONTROL_PLANE_TELEMETRY_PORT_OFFSET` 常量

### 5.4 i18n / UI 字符串增量

- `HpQuicPairingOpenBtn` "开放 Center 配对 5 分钟"
- `HpQuicPairingActive` "配对窗口剩余 {n}s"
- `HpQuicTrustedCentersHeader` "已信任的 Center"
- `HpQuicForgetCenter` "解除信任"
- `CenterTofuTitle` "首次连接：确认 Host 指纹"
- `CenterTofuFingerprintLabel` "SHA-256 指纹"
- `CenterTofuTrustBtn` "信任并连接"
- `CenterTofuCancelBtn` "取消"

---

## 6. Cargo 依赖增量

```toml
# crates/titan-quic/Cargo.toml
[dependencies]
quinn = { version = "0.11", default-features = false, features = ["runtime-tokio", "rustls-ring"] }
rustls = { version = "0.23", default-features = false, features = ["std", "ring"] }
rustls-pki-types = "1"
rcgen = { version = "0.13", default-features = false, features = ["pem", "ring"] }
ring = "0.17"
sha2 = "0.10"
hex = "0.4"
anyhow = "1"
tokio = { version = "1.43", features = ["rt", "sync", "io-util", "time"] }
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
titan-common = { path = "../titan-common" }

# apps/titan-host/Cargo.toml 与 apps/titan-center/Cargo.toml
titan-quic = { path = "../../crates/titan-quic" }
# 顺便：保留 rusqlite（已有，trust_store 复用）

# Cargo.lock 会因新增依赖产生增量，约 +30 个 transitive crates（quinn / ring / rcgen 链）
```

---

## 7. 测试计划

### 7.1 集成测试矩阵（替换旧 TCP 测试）

| 测试 | 验证 |
|---|---|
| `quic_hello.rs` | mTLS 握手 + Hello/HelloAck 全链路 |
| `quic_ping.rs` | Ping/Pong + 多 RPC 复用同一 connection |
| `quic_multi_rpc.rs` | 在同一 connection 上并发开 5 条 bi-stream，全部成功 |
| `quic_apply_vm_window_snapshot.rs` | `ApplyVmWindowSnapshot` 全链路 + ack |
| `quic_telemetry_subscribe.rs` | Subscribe → 收到第一条 push（用 mock host telemetry） |
| `quic_apply_spoof_profile.rs` | 现有 spoof 测试迁移 |

### 7.2 单元测试

| 模块 | 用例 |
|---|---|
| `titan-quic::identity` | 重启后从盘加载 = 一致 cert；fingerprint 稳定 |
| `titan-quic::endpoint` | 自定义 verifier：fingerprint 命中放行 / 不命中拒绝 |
| `apps/.../trust_store` | upsert / remove / contains 边界 |
| `host_app::pairing` | 窗口期到点关闭；首次接受后立即关闭 |
| `discovery::HostAnnounceBeacon` | schema v3 序列化 / v1+v2 兼容反序列化 |

### 7.3 手工验证清单（PR 描述里勾）

- [ ] Host 首启 → identity 文件生成；二次启动指纹不变
- [ ] Host 在 pairing window 内被 Center 添加 → 信任表写入 → 关闭后 Center 仍能连
- [ ] pairing window 关闭后未知 Center 连接 → 失败 + Host 日志可定位
- [ ] Center 删除 host 信任后再连接 → TOFU 弹窗
- [ ] 所有 ControlRequest 实操可达；telemetry 桌面 JPEG / 资源曲线正常
- [ ] `cargo fmt`、`RUSTFLAGS='-D warnings' cargo check --workspace`、`cargo clippy --workspace -- -D warnings`、
      `cargo test --workspace`、`python3 tools/check_fn_code_lines.py`、`./tools/check_rs_file_code_lines.sh` 全绿

---

## 8. 兼容性

新项目，**无历史包袱**：
- 不保留任何 TCP 兼容路径、不保留 `HostAnnounceBeacon` v1/v2 解析、不保留 `CONTROL_PLANE_TELEMETRY_PORT_OFFSET`。
- `PROTOCOL_VERSION = 16` 是首版数字，后续 wire 破坏性变更继续 +1。
- 所有现存的 `last_caps` 字符串、Persist JSON 结构里如出现 `host_control_addr` 字样，统一改名 `host_quic_addr`，
  并标注本次为 fresh schema。

---

## 9. 风险与边界

| 风险 | 处置 |
|---|---|
| `rcgen 0.13 + ed25519 + rustls 0.23` 跨平台编译 | 使用 `ring` backend（与 quinn `rustls-ring` 一致），避免 `aws-lc-rs` 链接问题 |
| 防火墙拦截 UDP 7788 | UI 加诊断按钮 "QUIC 自检"，跑一次 client→loopback 自连，失败时给出具体 OS 提示 |
| Pairing window 用户忘开 | UI 在 host 主面板高亮 "未信任 Center 尝试连接" 计数；Center 端弹错信息明确 |
| Trust store 误清空 | "解除信任" 加二次确认；备份提示 |
| rkyv 格式跨 PROTOCOL_VERSION | wire 类型枚举末位追加；仍按现有约定 (`backend-rules.mdc` 协议与版本) |
| 单 UDP 端口被占用 | `quinn::Endpoint::server` 失败时给出 i18n 错误并允许 UI 重新输入端口 |

---

## 10. 不在本次范围

- WebRTC / NVENC 推流（仍 Phase 2/3）
- guest agent 通讯（独立协议，`agent-bindings.toml` 路径，QUIC 化是后续工作）
- 0-RTT / connection migration / multipath（QUIC 高级特性，**不**在首版启用）
- 证书轮换、CRL：自签 + fingerprint TOFU 已覆盖目标威胁模型，无 PKI 需要

---

## 11. 上线 checklist（合并前）

- [x] 本文 review 通过（你确认架构与文件清单）
- [x] 实现完成、所有测试绿、手工验证清单全部勾选
- [x] `need.md` 第 3 行 Phase 段更新为 "Phase 2+：QUIC + mTLS 控制面 + Telemetry"
- [x] `requirements-traceability.md` 「Transport」行改写
- [x] 不新增 `Capabilities.quic_mtls_transport`：传输层固定 QUIC + mTLS，该能力位不再作为协商字段
- [x] PR 描述包含 Phase 升级声明、迁移说明、回滚预案（仅"还原 PR"，不维护双轨）
