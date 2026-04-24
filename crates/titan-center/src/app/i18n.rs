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

    DiscoveryTitle,
    DiscoveryUdpBlurb,
    DiscoveryCheckbox,
    DiscoveryBindBlurb,
    DiscoveryBindQuickAdd,
    DiscoveryBindScrollHint,
    DiscoveryRefreshIfaces,
    DiscoveryClearBindIps,
    DiscoveryNoIpv4Ifaces,
    IntervalLabel,
    UdpPortLabel,

    HostCollectTitle,
    HostCollectBlurb,
    HostCollectCheckbox,
    HostCollectIntervalLabel,
    HostCollectPollPortLabel,
    HostCollectRegisterPortLabel,

    /// Device management tab when there are no saved hosts (no section title).
    NoDataShort,
    /// Device management: centered headline when the registered list is empty.
    DeviceMgmtNoRegistered,
    /// Hint under empty device list: connect from Settings to populate.
    DeviceMgmtEmptyHint,
    /// Shown on device card preview until host desktop streaming is wired.
    DeviceMgmtDesktopPreviewNote,
    /// Device card: CPU usage label (value appended in UI).
    DeviceMgmtResCpu,
    /// Device card: memory usage label.
    DeviceMgmtResMem,
    /// Device card: network line; values are `down / up` (compact suffixes, no arrow glyphs).
    DeviceMgmtResNet,
    /// Device card: disk I/O line; values are `read / write` (compact suffixes, no arrow glyphs).
    DeviceMgmtResDiskIo,
    /// Device card: user remark section title.
    DeviceMgmtRemarkTitle,
    /// Device card: empty remark hint (double-click to edit).
    DeviceMgmtRemarkDblclkHint,
    BtnAddHost,
    /// Manual add-host dialog (IP + port).
    AddHostDialogTitle,
    AddHostDialogSubtitle,
    AddHostIpLabel,
    AddHostPortLabel,
    AddHostConfirm,
    AddHostInvalidHint,
    /// Add-host: Hello probe in progress (button disabled).
    AddHostVerifying,
    /// Toast when manual add-host Hello fails (offline / timeout / error).
    AddHostOfflineToast,
    /// Status log after manual add-host succeeds.
    AddHostSavedLog,
    BtnRemoveSelected,
    /// Device toolbar: send Hello to the currently selected host.
    BtnHostHello,
    /// Device toolbar: open telemetry stream for the selected host.
    BtnHostTelemetry,

    VmInventoryTitle,
    ColState,
    /// VM tile second line: "Host · {device label}".
    VmTileHostPrefix,

    WindowPreviewTitle,
    WindowPreviewHint,

    MonitorCardDevices,
    MonitorCardWindows,
    MonitorStatTotal,
    MonitorStatOnline,
    MonitorStatOffline,
    MonitorDevicesScopeHint,
    MonitorWindowsScopeHint,

    /// Second column title on wide layouts (Spoof / Power), aligned with Usage cards.
    CardActions,

    SlotGridTitle,
    SlotEmpty,
    NoHost,

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

        (UiLang::En, Msg::NavConnect) => "Devices",
        (UiLang::Zh, Msg::NavConnect) => "设备管理",
        (UiLang::En, Msg::NavSettings) => "Settings",
        (UiLang::Zh, Msg::NavSettings) => "设置",
        (UiLang::En, Msg::NavHostsVms) => "Windows",
        (UiLang::Zh, Msg::NavHostsVms) => "窗口管理",
        (UiLang::En, Msg::NavMonitor) => "Usage",
        (UiLang::Zh, Msg::NavMonitor) => "资源监控",
        (UiLang::En, Msg::NavSpoof) => "Spoof",
        (UiLang::Zh, Msg::NavSpoof) => "主机伪装",
        (UiLang::En, Msg::NavPower) => "Power",
        (UiLang::Zh, Msg::NavPower) => "批量电源",

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
        (UiLang::En, Msg::DiscoveryBindBlurb) => {
            "Optional: pick one or more local IPv4 addresses to send subnet broadcasts from (multi-homed). Leave none selected to use the OS default route (255.255.255.255)."
        }
        (UiLang::Zh, Msg::DiscoveryBindBlurb) => {
            "可选：选择一个或多个本机 IPv4，从对应网卡向子网广播；不选则走系统默认路由（255.255.255.255）。"
        }
        (UiLang::En, Msg::DiscoveryBindQuickAdd) => "Add interface IPv4…",
        (UiLang::Zh, Msg::DiscoveryBindQuickAdd) => "从列表添加 IPv4…",
        (UiLang::En, Msg::DiscoveryBindScrollHint) => "Multi-select (non-loopback IPv4):",
        (UiLang::Zh, Msg::DiscoveryBindScrollHint) => "多选广播源（不含回环）：",
        (UiLang::En, Msg::DiscoveryRefreshIfaces) => "Refresh interfaces",
        (UiLang::Zh, Msg::DiscoveryRefreshIfaces) => "刷新网卡列表",
        (UiLang::En, Msg::DiscoveryClearBindIps) => "Clear selection",
        (UiLang::Zh, Msg::DiscoveryClearBindIps) => "清空已选",
        (UiLang::En, Msg::DiscoveryNoIpv4Ifaces) => "No non-loopback IPv4 interfaces found.",
        (UiLang::Zh, Msg::DiscoveryNoIpv4Ifaces) => "未发现可用的非回环 IPv4 网卡。",
        (UiLang::En, Msg::IntervalLabel) => "Interval (s):",
        (UiLang::Zh, Msg::IntervalLabel) => "间隔（秒）：",
        (UiLang::En, Msg::UdpPortLabel) => "UDP port:",
        (UiLang::Zh, Msg::UdpPortLabel) => "UDP 端口：",

        (UiLang::En, Msg::HostCollectTitle) => "LAN host registration (scale-out)",
        (UiLang::Zh, Msg::HostCollectTitle) => "局域网宿主注册（批量）",
        (UiLang::En, Msg::HostCollectBlurb) => {
            "Broadcasts a short poll on UDP. Each `titan-host serve` on the LAN replies with its control-plane TCP address — no per-host Connect click. Uses the same bind-IPv4 list as guest discovery when set."
        }
        (UiLang::Zh, Msg::HostCollectBlurb) => {
            "在局域网周期性广播唤请包（UDP）。各台 `titan-host serve` 收到后自动向中控上报控制面 TCP 地址，无需在每台机器上单独操作中控。若上方指定了绑定 IPv4，则与本页「来宾发现」共用该列表。"
        }
        (UiLang::En, Msg::HostCollectCheckbox) => "Broadcast registration poll",
        (UiLang::Zh, Msg::HostCollectCheckbox) => "广播唤请宿主注册",
        (UiLang::En, Msg::HostCollectIntervalLabel) => "Poll interval (seconds):",
        (UiLang::Zh, Msg::HostCollectIntervalLabel) => "唤请间隔（秒）：",
        (UiLang::En, Msg::HostCollectPollPortLabel) => "Poll UDP port (destination):",
        (UiLang::Zh, Msg::HostCollectPollPortLabel) => "唤请 UDP 端口（目的）：",
        (UiLang::En, Msg::HostCollectRegisterPortLabel) => "Host reply UDP port (center listens):",
        (UiLang::Zh, Msg::HostCollectRegisterPortLabel) => "宿主应答 UDP 端口（中控监听）：",

        (UiLang::En, Msg::NoDataShort) => "No data",
        (UiLang::Zh, Msg::NoDataShort) => "暂无数据",
        (UiLang::En, Msg::DeviceMgmtNoRegistered) => "No registered devices yet",
        (UiLang::Zh, Msg::DeviceMgmtNoRegistered) => "暂无注册设备～",
        (UiLang::En, Msg::DeviceMgmtEmptyHint) => {
            "No hosts yet. Use **Settings** → LAN discovery / LAN host registration; devices also merge here when they announce on UDP."
        }
        (UiLang::Zh, Msg::DeviceMgmtEmptyHint) => {
            "暂无设备。请打开左侧「设置」使用局域网发现或局域网宿主注册；宿主 UDP 宣告后也会出现在此列表。"
        }
        (UiLang::En, Msg::DeviceMgmtDesktopPreviewNote) => {
            "Desktop preview — live capture not connected yet"
        }
        (UiLang::Zh, Msg::DeviceMgmtDesktopPreviewNote) => {
            "桌面预览 — 尚未接入实时画面"
        }
        (UiLang::En, Msg::DeviceMgmtResCpu) => "CPU",
        (UiLang::Zh, Msg::DeviceMgmtResCpu) => "CPU",
        (UiLang::En, Msg::DeviceMgmtResMem) => "Mem",
        (UiLang::Zh, Msg::DeviceMgmtResMem) => "内存",
        (UiLang::En, Msg::DeviceMgmtResNet) => "Net",
        (UiLang::Zh, Msg::DeviceMgmtResNet) => "网速",
        (UiLang::En, Msg::DeviceMgmtResDiskIo) => "Disk",
        (UiLang::Zh, Msg::DeviceMgmtResDiskIo) => "磁盘",
        (UiLang::En, Msg::DeviceMgmtRemarkTitle) => "Note",
        (UiLang::Zh, Msg::DeviceMgmtRemarkTitle) => "备注",
        (UiLang::En, Msg::DeviceMgmtRemarkDblclkHint) => {
            "Double-click to add a note…"
        },
        (UiLang::Zh, Msg::DeviceMgmtRemarkDblclkHint) => "双击添加备注…",
        (UiLang::En, Msg::BtnAddHost) => "+ Add host",
        (UiLang::Zh, Msg::BtnAddHost) => "+ 添加主机",
        (UiLang::En, Msg::AddHostDialogTitle) => "Add host",
        (UiLang::Zh, Msg::AddHostDialogTitle) => "添加主机",
        (UiLang::En, Msg::AddHostDialogSubtitle) => {
            "Control-plane address on the machine running `titan-host serve`. The host must be online — we send Hello before adding."
        }
        (UiLang::Zh, Msg::AddHostDialogSubtitle) => {
            "填写运行 `titan-host serve` 的机器上的控制面 TCP 地址。添加前会检测在线（发送 Hello）。"
        }
        (UiLang::En, Msg::AddHostIpLabel) => "IPv4 address",
        (UiLang::Zh, Msg::AddHostIpLabel) => "IPv4 地址",
        (UiLang::En, Msg::AddHostPortLabel) => "TCP port",
        (UiLang::Zh, Msg::AddHostPortLabel) => "TCP 端口",
        (UiLang::En, Msg::AddHostConfirm) => "Add",
        (UiLang::Zh, Msg::AddHostConfirm) => "添加",
        (UiLang::En, Msg::AddHostInvalidHint) => {
            "Enter a valid IPv4 and a port from 1 to 65535."
        }
        (UiLang::Zh, Msg::AddHostInvalidHint) => "请输入有效的 IPv4 与 1–65535 范围内的端口。",
        (UiLang::En, Msg::AddHostVerifying) => "Checking…",
        (UiLang::Zh, Msg::AddHostVerifying) => "正在检测…",
        (UiLang::En, Msg::AddHostOfflineToast) => "Device is offline.",
        (UiLang::Zh, Msg::AddHostOfflineToast) => "设备不在线～",
        (UiLang::En, Msg::AddHostSavedLog) => "Host added (online check OK).",
        (UiLang::Zh, Msg::AddHostSavedLog) => "已添加主机（在线检测通过）。",
        (UiLang::En, Msg::BtnRemoveSelected) => "Remove selected",
        (UiLang::Zh, Msg::BtnRemoveSelected) => "删除选中",
        (UiLang::En, Msg::BtnHostHello) => "Hello",
        (UiLang::Zh, Msg::BtnHostHello) => "Hello",
        (UiLang::En, Msg::BtnHostTelemetry) => "Telemetry",
        (UiLang::Zh, Msg::BtnHostTelemetry) => "遥测",

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

        (UiLang::En, Msg::MonitorCardDevices) => "Devices",
        (UiLang::Zh, Msg::MonitorCardDevices) => "设备数量",
        (UiLang::En, Msg::MonitorCardWindows) => "Windows",
        (UiLang::Zh, Msg::MonitorCardWindows) => "窗口数",
        (UiLang::En, Msg::MonitorStatTotal) => "Total",
        (UiLang::Zh, Msg::MonitorStatTotal) => "总数",
        (UiLang::En, Msg::MonitorStatOnline) => "Online",
        (UiLang::Zh, Msg::MonitorStatOnline) => "在线",
        (UiLang::En, Msg::MonitorStatOffline) => "Offline",
        (UiLang::Zh, Msg::MonitorStatOffline) => "离线",
        (UiLang::En, Msg::MonitorDevicesScopeHint) => {
            "Online = last successful connect (Hello/Ping) while this device row was selected."
        }
        (UiLang::Zh, Msg::MonitorDevicesScopeHint) => {
            "在线=该行设备为当前会话目标且最近一次连接成功（Hello/Ping）。"
        }
        (UiLang::En, Msg::MonitorWindowsScopeHint) => {
            "Running vs other states from the last List VMs on the selected device."
        }
        (UiLang::Zh, Msg::MonitorWindowsScopeHint) => {
            "在线=运行中；统计来自当前选中设备最近一次「列出虚拟机」。"
        }
        (UiLang::En, Msg::CardActions) => "Actions",
        (UiLang::Zh, Msg::CardActions) => "操作",

        (UiLang::En, Msg::SlotGridTitle) => "Slot grid (virtualized)",
        (UiLang::Zh, Msg::SlotGridTitle) => "槽位网格（虚拟化示例）",
        (UiLang::En, Msg::SlotEmpty) => "empty",
        (UiLang::Zh, Msg::SlotEmpty) => "空",
        (UiLang::En, Msg::NoHost) => "no-host",
        (UiLang::Zh, Msg::NoHost) => "未选主机",

        (UiLang::En, Msg::SpoofCardTitle) => "Host spoof (control plane)",
        (UiLang::Zh, Msg::SpoofCardTitle) => "宿主伪装（控制面）",
        (UiLang::En, Msg::SpoofBlurb) => {
            "Host session must be ready first. Preview = dry-run; Apply runs Hyper-V PowerShell on the host (EULA / law is your responsibility)."
        }
        (UiLang::Zh, Msg::SpoofBlurb) => {
            "请先等待宿主会话就绪。预览为 dry-run；应用会在宿主执行 Hyper-V PowerShell（EULA/法律自负）。"
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
pub fn log_lan_host_announced(lang: UiLang, label: &str, addr: &str) -> String {
    match lang {
        UiLang::En => format!("LAN: host {label} announced at {addr} (device list updated)."),
        UiLang::Zh => format!("局域网：主机 {label} 在 {addr} 宣告（设备列表已更新）。"),
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
