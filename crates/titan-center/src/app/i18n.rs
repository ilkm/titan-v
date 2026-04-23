//! UI strings (English / Chinese). Extend `UiLang` + `t` match arms for more locales.

use serde::{Deserialize, Serialize};

/// Display language for the center UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLang {
    #[default]
    En,
    Zh,
}

/// Static UI copy keys.
#[derive(Clone, Copy)]
pub enum Msg {
    BrandTitle,
    SettingsTooltip,
    SettingsTitle,
    SettingsClose,
    SettingsMoreLangNote,
    LangRadioEn,
    LangRadioZh,

    NavConnect,
    NavSettings,
    NavHostsVms,
    NavMonitor,
    NavSpoof,
    NavPower,

    ConnectionCardTitle,
    ConnectionM2Hint,
    HintControlAddr,
    BtnSyncHost,
    BtnListVms,
    ChkAutoRefresh,
    PollIntervalLabel,

    DiscoveryTitle,
    DiscoveryUdpBlurb,
    DiscoveryCheckbox,
    IntervalLabel,
    UdpPortLabel,

    GuestCardTitle,
    GuestBlurb,
    VmLabelSmall,
    AgentLabelSmall,
    HintVmName,
    HintAgentAddr,
    BtnRegisterHost,

    SessionTitle,
    BtnConnect,
    BtnDisconnect,
    BtnPing,
    WaitEllipsis,
    StatusConnected,
    StatusNotConnected,
    CapsSnapshotTitle,

    HostsCardTitle,
    HostListEmpty,
    HostListSelectedEditTitle,
    HostListNameField,
    HostListAddrField,
    DeviceMgmtNavHint,
    /// Device management tab when there are no saved hosts (no section title).
    NoDataShort,
    BtnAddHost,
    BtnRemoveSelected,

    VmInventoryTitle,
    ColState,
    /// VM tile second line: "Host · {device label}".
    VmTileHostPrefix,

    WindowPreviewTitle,
    WindowPreviewHint,

    StatusBoardTitle,
    AccountsCardTitle,
    BtnAccount,
    BtnProxyLabel,
    AccountsLabel,
    ProxiesLabel,
    ScriptArtifactHint,
    HintScriptVersion,

    SlotGridTitle,
    SlotEmpty,
    NoHost,

    ActivityTitle,
    ActivityHint,

    SpoofCardTitle,
    SpoofBlurb,
    TargetVmLabel,
    HintTargetVm,
    ChkDynamicMac,
    ChkDisableCkpt,
    BtnPreviewDryRun,
    BtnApplyEllipsis,
    WinConfirmSpoofTitle,
    WinConfirmSpoofBody,
    BtnCancel,
    BtnConfirmApply,

    DangerCardTitle,
    DangerBlurb,
    HintBulkVms,
    BtnBulkStart,
    BtnBulkStop,
    WinConfirmStopTitle,
    WinConfirmStopBody,
    BtnConfirmStop,
    WinConfirmStartTitle,
    WinConfirmStartBody,
    BtnConfirmStart,
}

#[must_use]
pub fn t(lang: UiLang, msg: Msg) -> &'static str {
    match (lang, msg) {
        (UiLang::En, Msg::BrandTitle) => "Titan Center",
        (UiLang::Zh, Msg::BrandTitle) => "Titan 中控",
        (UiLang::En, Msg::SettingsTooltip) => "Settings",
        (UiLang::Zh, Msg::SettingsTooltip) => "设置",
        (UiLang::En, Msg::SettingsTitle) => "Settings",
        (UiLang::Zh, Msg::SettingsTitle) => "设置",
        (UiLang::En, Msg::SettingsClose) => "Close",
        (UiLang::Zh, Msg::SettingsClose) => "关闭",
        (UiLang::En, Msg::SettingsMoreLangNote) => {
            "More languages can be added here later (e.g. locale files)."
        }
        (UiLang::Zh, Msg::SettingsMoreLangNote) => "后续可在此加载更多语言（例如独立语言包）。",
        (UiLang::En, Msg::LangRadioEn) => "English",
        (UiLang::Zh, Msg::LangRadioEn) => "English",
        (UiLang::En, Msg::LangRadioZh) => "中文",
        (UiLang::Zh, Msg::LangRadioZh) => "中文",

        (UiLang::En, Msg::NavConnect) => "Device management",
        (UiLang::Zh, Msg::NavConnect) => "设备管理",
        (UiLang::En, Msg::NavSettings) => "Settings",
        (UiLang::Zh, Msg::NavSettings) => "设置",
        (UiLang::En, Msg::NavHostsVms) => "Window management",
        (UiLang::Zh, Msg::NavHostsVms) => "窗口管理",
        (UiLang::En, Msg::NavMonitor) => "Resource monitor",
        (UiLang::Zh, Msg::NavMonitor) => "资源监控",
        (UiLang::En, Msg::NavSpoof) => "Host spoof",
        (UiLang::Zh, Msg::NavSpoof) => "主机伪装",
        (UiLang::En, Msg::NavPower) => "Bulk power",
        (UiLang::Zh, Msg::NavPower) => "批量电源",

        (UiLang::En, Msg::ConnectionCardTitle) => "Connection & inventory",
        (UiLang::Zh, Msg::ConnectionCardTitle) => "连接与清单",
        (UiLang::En, Msg::ConnectionM2Hint) => "M2 control address (titan-host serve)",
        (UiLang::Zh, Msg::ConnectionM2Hint) => "M2 控制地址（titan-host serve）",
        (UiLang::En, Msg::HintControlAddr) => "e.g. 192.168.1.10:7788",
        (UiLang::Zh, Msg::HintControlAddr) => "例如 192.168.1.10:7788",
        (UiLang::En, Msg::BtnSyncHost) => "Sync from selected host",
        (UiLang::Zh, Msg::BtnSyncHost) => "从选中主机同步",
        (UiLang::En, Msg::BtnListVms) => "List VMs",
        (UiLang::Zh, Msg::BtnListVms) => "列出虚拟机",
        (UiLang::En, Msg::ChkAutoRefresh) => "Auto refresh VM list when connected",
        (UiLang::Zh, Msg::ChkAutoRefresh) => "已连接时自动刷新虚拟机列表",
        (UiLang::En, Msg::PollIntervalLabel) => "Poll interval (s):",
        (UiLang::Zh, Msg::PollIntervalLabel) => "轮询间隔（秒）：",

        (UiLang::En, Msg::DiscoveryTitle) => "LAN discovery (optional)",
        (UiLang::Zh, Msg::DiscoveryTitle) => "局域网发现（可选）",
        (UiLang::En, Msg::DiscoveryUdpBlurb) => {
            "UDP broadcast of `DiscoveryBeacon` so in-guest automation can learn this control address."
        }
        (UiLang::Zh, Msg::DiscoveryUdpBlurb) => {
            "通过 UDP 广播 `DiscoveryBeacon`，便于来宾内自动化获知本控制地址。"
        }
        (UiLang::En, Msg::DiscoveryCheckbox) => "Broadcast discovery on LAN",
        (UiLang::Zh, Msg::DiscoveryCheckbox) => "在局域网广播发现",
        (UiLang::En, Msg::IntervalLabel) => "Interval (s):",
        (UiLang::Zh, Msg::IntervalLabel) => "间隔（秒）：",
        (UiLang::En, Msg::UdpPortLabel) => "UDP port:",
        (UiLang::Zh, Msg::UdpPortLabel) => "UDP 端口：",

        (UiLang::En, Msg::GuestCardTitle) => "Guest agent binding",
        (UiLang::Zh, Msg::GuestCardTitle) => "来宾代理绑定",
        (UiLang::En, Msg::GuestBlurb) => {
            "After Connect: map Hyper-V VM name → guest agent TCP address (reachable from the host)."
        }
        (UiLang::Zh, Msg::GuestBlurb) => {
            "连接后：将 Hyper-V 虚拟机名映射为来宾代理 TCP 地址（须从宿主可达）。"
        }
        (UiLang::En, Msg::VmLabelSmall) => "VM",
        (UiLang::Zh, Msg::VmLabelSmall) => "虚拟机",
        (UiLang::En, Msg::AgentLabelSmall) => "Agent",
        (UiLang::Zh, Msg::AgentLabelSmall) => "代理",
        (UiLang::En, Msg::HintVmName) => "name",
        (UiLang::Zh, Msg::HintVmName) => "名称",
        (UiLang::En, Msg::HintAgentAddr) => "host-visible IP:port",
        (UiLang::Zh, Msg::HintAgentAddr) => "宿主可见 IP:端口",
        (UiLang::En, Msg::BtnRegisterHost) => "Register on host",
        (UiLang::Zh, Msg::BtnRegisterHost) => "在宿主注册",

        (UiLang::En, Msg::SessionTitle) => "Session",
        (UiLang::Zh, Msg::SessionTitle) => "会话",
        (UiLang::En, Msg::BtnConnect) => "Connect",
        (UiLang::Zh, Msg::BtnConnect) => "连接",
        (UiLang::En, Msg::BtnDisconnect) => "Disconnect",
        (UiLang::Zh, Msg::BtnDisconnect) => "断开",
        (UiLang::En, Msg::BtnPing) => "Ping",
        (UiLang::Zh, Msg::BtnPing) => "Ping",
        (UiLang::En, Msg::WaitEllipsis) => "…",
        (UiLang::Zh, Msg::WaitEllipsis) => "…",
        (UiLang::En, Msg::StatusConnected) => "● Connected",
        (UiLang::Zh, Msg::StatusConnected) => "● 已连接",
        (UiLang::En, Msg::StatusNotConnected) => "○ Not connected — use Connect (Hello)",
        (UiLang::Zh, Msg::StatusNotConnected) => "○ 未连接 — 请先点「连接」(Hello)",
        (UiLang::En, Msg::CapsSnapshotTitle) => "Capabilities snapshot",
        (UiLang::Zh, Msg::CapsSnapshotTitle) => "能力快照",

        (UiLang::En, Msg::HostsCardTitle) => "Host devices",
        (UiLang::Zh, Msg::HostsCardTitle) => "宿主机设备",
        (UiLang::En, Msg::HostListEmpty) => "No host devices yet — use + Add host below.",
        (UiLang::Zh, Msg::HostListEmpty) => "暂无数据 — 请使用下方「添加主机」。",
        (UiLang::En, Msg::HostListSelectedEditTitle) => "Edit selected device",
        (UiLang::Zh, Msg::HostListSelectedEditTitle) => "编辑当前选中的设备",
        (UiLang::En, Msg::HostListNameField) => "Device name",
        (UiLang::Zh, Msg::HostListNameField) => "设备名称",
        (UiLang::En, Msg::HostListAddrField) => "Control IP:port",
        (UiLang::Zh, Msg::HostListAddrField) => "控制地址 IP:端口",
        (UiLang::En, Msg::DeviceMgmtNavHint) => {
            "Connection, discovery, host list, and session controls are under Settings in the left nav."
        }
        (UiLang::Zh, Msg::DeviceMgmtNavHint) => {
            "连接、发现、宿主机列表与会话控制已整合到左侧导航的「设置」页面。"
        }
        (UiLang::En, Msg::NoDataShort) => "No data",
        (UiLang::Zh, Msg::NoDataShort) => "暂无数据",
        (UiLang::En, Msg::BtnAddHost) => "+ Add host",
        (UiLang::Zh, Msg::BtnAddHost) => "+ 添加主机",
        (UiLang::En, Msg::BtnRemoveSelected) => "Remove selected",
        (UiLang::Zh, Msg::BtnRemoveSelected) => "删除选中",

        (UiLang::En, Msg::VmInventoryTitle) => "VM inventory",
        (UiLang::Zh, Msg::VmInventoryTitle) => "虚拟机清单",
        (UiLang::En, Msg::ColState) => "State",
        (UiLang::Zh, Msg::ColState) => "状态",
        (UiLang::En, Msg::VmTileHostPrefix) => "Host",
        (UiLang::Zh, Msg::VmTileHostPrefix) => "宿主",

        (UiLang::En, Msg::WindowPreviewTitle) => "VM preview",
        (UiLang::Zh, Msg::WindowPreviewTitle) => "画面预览",
        (UiLang::En, Msg::WindowPreviewHint) => {
            "Placeholder for per-window video (host · VM · slot 1–40 wiring comes later)."
        }
        (UiLang::Zh, Msg::WindowPreviewHint) => {
            "视频预览占位。后续按「宿主机名 · 虚拟机 · 窗口编号 1–40」接入推流。"
        }

        (UiLang::En, Msg::StatusBoardTitle) => "Status board",
        (UiLang::Zh, Msg::StatusBoardTitle) => "状态看板",
        (UiLang::En, Msg::AccountsCardTitle) => "Accounts & proxies (labels)",
        (UiLang::Zh, Msg::AccountsCardTitle) => "账号与代理（标签）",
        (UiLang::En, Msg::BtnAccount) => "+ Account",
        (UiLang::Zh, Msg::BtnAccount) => "+ 账号",
        (UiLang::En, Msg::BtnProxyLabel) => "+ Proxy label",
        (UiLang::Zh, Msg::BtnProxyLabel) => "+ 代理标签",
        (UiLang::En, Msg::AccountsLabel) => "Accounts",
        (UiLang::Zh, Msg::AccountsLabel) => "账号",
        (UiLang::En, Msg::ProxiesLabel) => "Proxies",
        (UiLang::Zh, Msg::ProxiesLabel) => "代理",
        (UiLang::En, Msg::ScriptArtifactHint) => {
            "Script artifact version label (optional mirror for SetScriptArtifact)"
        }
        (UiLang::Zh, Msg::ScriptArtifactHint) => {
            "脚本产物版本标签（可选，与 SetScriptArtifact 对应）"
        }
        (UiLang::En, Msg::HintScriptVersion) => "e.g. v1.2.3",
        (UiLang::Zh, Msg::HintScriptVersion) => "例如 v1.2.3",

        (UiLang::En, Msg::SlotGridTitle) => "Slot grid (virtualized)",
        (UiLang::Zh, Msg::SlotGridTitle) => "槽位网格（虚拟化示例）",
        (UiLang::En, Msg::SlotEmpty) => "empty",
        (UiLang::Zh, Msg::SlotEmpty) => "空",
        (UiLang::En, Msg::NoHost) => "no-host",
        (UiLang::Zh, Msg::NoHost) => "未选主机",

        (UiLang::En, Msg::ActivityTitle) => "Activity",
        (UiLang::Zh, Msg::ActivityTitle) => "活动",
        (UiLang::En, Msg::ActivityHint) => {
            "Connect to a host, then List VMs or use actions on the left."
        }
        (UiLang::Zh, Msg::ActivityHint) => "请先连接宿主，再列出虚拟机或使用左侧操作。",

        (UiLang::En, Msg::SpoofCardTitle) => "Host spoof (M2)",
        (UiLang::Zh, Msg::SpoofCardTitle) => "宿主伪装 (M2)",
        (UiLang::En, Msg::SpoofBlurb) => {
            "Connect first. Preview = dry-run; Apply runs Hyper-V PowerShell on the host (EULA / law is your responsibility)."
        }
        (UiLang::Zh, Msg::SpoofBlurb) => {
            "请先连接。预览为 dry-run；应用会在宿主执行 Hyper-V PowerShell（EULA/法律自负）。"
        }
        (UiLang::En, Msg::TargetVmLabel) => "Target VM",
        (UiLang::Zh, Msg::TargetVmLabel) => "目标虚拟机",
        (UiLang::En, Msg::HintTargetVm) => "Hyper-V VM name",
        (UiLang::Zh, Msg::HintTargetVm) => "Hyper-V 虚拟机名",
        (UiLang::En, Msg::ChkDynamicMac) => {
            "Dynamic MAC (Set-VMNetworkAdapter -DynamicMacAddress On)"
        }
        (UiLang::Zh, Msg::ChkDynamicMac) => "动态 MAC（Set-VMNetworkAdapter -DynamicMacAddress On）",
        (UiLang::En, Msg::ChkDisableCkpt) => "Disable checkpoints (Set-VM -CheckpointType Disabled)",
        (UiLang::Zh, Msg::ChkDisableCkpt) => "禁用检查点（Set-VM -CheckpointType Disabled）",
        (UiLang::En, Msg::BtnPreviewDryRun) => "Preview (dry-run)",
        (UiLang::Zh, Msg::BtnPreviewDryRun) => "预览（dry-run）",
        (UiLang::En, Msg::BtnApplyEllipsis) => "Apply…",
        (UiLang::Zh, Msg::BtnApplyEllipsis) => "应用…",
        (UiLang::En, Msg::WinConfirmSpoofTitle) => "Confirm spoof apply",
        (UiLang::Zh, Msg::WinConfirmSpoofTitle) => "确认应用伪装",
        (UiLang::En, Msg::WinConfirmSpoofBody) => {
            "This sends ApplySpoofProfile with dry_run=false to the connected host."
        }
        (UiLang::Zh, Msg::WinConfirmSpoofBody) => {
            "将向已连接的宿主发送 ApplySpoofProfile，且 dry_run=false。"
        }
        (UiLang::En, Msg::BtnCancel) => "Cancel",
        (UiLang::Zh, Msg::BtnCancel) => "取消",
        (UiLang::En, Msg::BtnConfirmApply) => "Confirm apply",
        (UiLang::Zh, Msg::BtnConfirmApply) => "确认应用",

        (UiLang::En, Msg::DangerCardTitle) => "Bulk power",
        (UiLang::Zh, Msg::DangerCardTitle) => "批量电源",
        (UiLang::En, Msg::DangerBlurb) => {
            "Comma-separated VM names → StartVmGroup / StopVmGroup on the current control address."
        }
        (UiLang::Zh, Msg::DangerBlurb) => {
            "逗号分隔的虚拟机名 → 对当前控制地址执行 StartVmGroup / StopVmGroup。"
        }
        (UiLang::En, Msg::HintBulkVms) => "vm-01, vm-02, …",
        (UiLang::Zh, Msg::HintBulkVms) => "vm-01, vm-02, …",
        (UiLang::En, Msg::BtnBulkStart) => "Bulk start…",
        (UiLang::Zh, Msg::BtnBulkStart) => "批量启动…",
        (UiLang::En, Msg::BtnBulkStop) => "Bulk stop…",
        (UiLang::Zh, Msg::BtnBulkStop) => "批量停止…",
        (UiLang::En, Msg::WinConfirmStopTitle) => "Confirm bulk stop",
        (UiLang::Zh, Msg::WinConfirmStopTitle) => "确认批量停止",
        (UiLang::En, Msg::WinConfirmStopBody) => "StopVmGroup will run for the VM names above.",
        (UiLang::Zh, Msg::WinConfirmStopBody) => "将对上述虚拟机名执行 StopVmGroup。",
        (UiLang::En, Msg::BtnConfirmStop) => "Confirm stop",
        (UiLang::Zh, Msg::BtnConfirmStop) => "确认停止",
        (UiLang::En, Msg::WinConfirmStartTitle) => "Confirm bulk start",
        (UiLang::Zh, Msg::WinConfirmStartTitle) => "确认批量启动",
        (UiLang::En, Msg::WinConfirmStartBody) => "StartVmGroup will run for the VM names above.",
        (UiLang::Zh, Msg::WinConfirmStartBody) => "将对上述虚拟机名执行 StartVmGroup。",
        (UiLang::En, Msg::BtnConfirmStart) => "Confirm start",
        (UiLang::Zh, Msg::BtnConfirmStart) => "确认启动",
    }
}

#[must_use]
pub fn host_running_windows_line(lang: UiLang, n: u32) -> String {
    match lang {
        UiLang::En => format!("{n} running windows"),
        UiLang::Zh => format!("{n} 个运行窗口"),
    }
}

#[must_use]
pub fn log_host_responded(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Host responded (Hello or Ping).".into(),
        UiLang::Zh => "宿主已响应（Hello 或 Ping）。".into(),
    }
}

#[must_use]
pub fn log_list_vms(lang: UiLang, n: usize) -> String {
    match lang {
        UiLang::En => format!("ListVms: {n} VM(s)."),
        UiLang::Zh => format!("ListVms：{n} 台虚拟机。"),
    }
}

#[must_use]
pub fn log_stop_vm_group(lang: UiLang, ok: u32, failures: usize) -> String {
    match lang {
        UiLang::En => format!("StopVmGroup: ok={ok}, failures={failures}"),
        UiLang::Zh => format!("StopVmGroup：成功 {ok}，失败 {failures}"),
    }
}

#[must_use]
pub fn log_start_vm_group(lang: UiLang, ok: u32, failures: usize) -> String {
    match lang {
        UiLang::En => format!("StartVmGroup: ok={ok}, failures={failures}"),
        UiLang::Zh => format!("StartVmGroup：成功 {ok}，失败 {failures}"),
    }
}

#[must_use]
pub fn log_spoof_apply(lang: UiLang, dry_run: bool, steps: &str, notes: &str) -> String {
    match lang {
        UiLang::En => format!("ApplySpoofProfile dry_run={dry_run}: {steps} | {notes}"),
        UiLang::Zh => format!("ApplySpoofProfile dry_run={dry_run}：{steps} | {notes}"),
    }
}

#[must_use]
pub fn log_guest_reg(lang: UiLang, vm: &str) -> String {
    match lang {
        UiLang::En => format!("RegisterGuestAgent ok for VM {vm} (host saved binding)."),
        UiLang::Zh => format!("RegisterGuestAgent 成功：{vm}（宿主已保存绑定）。"),
    }
}

#[must_use]
pub fn log_request_failed(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Request failed.".into(),
        UiLang::Zh => "请求失败。".into(),
    }
}

#[must_use]
pub fn log_disconnected(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Disconnected.".into(),
        UiLang::Zh => "已断开。".into(),
    }
}

#[must_use]
pub fn log_spoof_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Spoof apply cancelled.".into(),
        UiLang::Zh => "已取消应用伪装。".into(),
    }
}

#[must_use]
pub fn log_spoof_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "ApplySpoofProfile dispatched.".into(),
        UiLang::Zh => "已发送 ApplySpoofProfile。".into(),
    }
}

#[must_use]
pub fn log_bulk_stop_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Bulk stop cancelled.".into(),
        UiLang::Zh => "已取消批量停止。".into(),
    }
}

#[must_use]
pub fn log_bulk_start_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Bulk start cancelled.".into(),
        UiLang::Zh => "已取消批量启动。".into(),
    }
}

#[must_use]
pub fn log_no_vm_names(lang: UiLang) -> String {
    match lang {
        UiLang::En => "No VM names parsed (comma-separated).".into(),
        UiLang::Zh => "未解析到虚拟机名（请用逗号分隔）。".into(),
    }
}

#[must_use]
pub fn log_stop_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "StopVmGroup dispatched.".into(),
        UiLang::Zh => "已发送 StopVmGroup。".into(),
    }
}

#[must_use]
pub fn log_start_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "StartVmGroup dispatched.".into(),
        UiLang::Zh => "已发送 StartVmGroup。".into(),
    }
}

#[must_use]
pub fn fmt_status_board_stats(lang: UiLang, vms: usize, accounts: usize, proxies: usize) -> String {
    match lang {
        UiLang::En => format!("VMs: {vms} · accounts: {accounts} · proxy labels: {proxies}"),
        UiLang::Zh => format!("虚拟机：{vms} · 账号：{accounts} · 代理标签：{proxies}"),
    }
}

#[must_use]
pub fn fmt_slot_grid_header(lang: UiLang, rows: usize, host: &str) -> String {
    match lang {
        UiLang::En => format!("{rows} rows · selected host: {host}"),
        UiLang::Zh => format!("{rows} 行 · 当前主机：{host}"),
    }
}

#[must_use]
pub fn fmt_slot_line_empty(lang: UiLang, host: &str, i: usize) -> String {
    let empty = t(lang, Msg::SlotEmpty);
    format!("{host} · {i:04} — {empty}")
}

#[must_use]
pub fn fmt_slot_line_vm(
    _lang: UiLang,
    host: &str,
    i: usize,
    vm_name: &str,
    state_dbg: &str,
) -> String {
    format!("{host} · {i:04} → {vm_name} · {state_dbg}")
}
