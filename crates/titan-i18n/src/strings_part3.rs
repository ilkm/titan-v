use crate::UiLang;
use crate::msg::Msg;

const CENTER_TOFU_DIALOG_SUBTITLE_EN: &str = concat!(
    "This host is not in the trust store yet. Verify the fingerprint matches ",
    "the one shown on the Titan Host settings panel before continuing."
);

const CENTER_TOFU_WARNING_EN: &str = concat!(
    "Trusting an unknown fingerprint exposes this control plane to MITM. ",
    "Confirm only when the value matches the fingerprint shown on the host machine."
);

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    center_vm_window_create_i18n(lang, msg)
        .or_else(|| inventory_and_preview(lang, msg))
        .or_else(|| monitor_counts(lang, msg))
        .or_else(|| monitor_hints_and_actions(lang, msg))
        .or_else(|| tofu_dialog(lang, msg))
        .or_else(|| center_settings_mtls(lang, msg))
        .or_else(|| tray(lang, msg))
}

fn tofu_dialog(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::CenterTofuDialogTitle) => Some("Trust this host?"),
        (UiLang::Zh, Msg::CenterTofuDialogTitle) => Some("信任该宿主机？"),
        (UiLang::En, Msg::CenterTofuDialogSubtitle) => Some(CENTER_TOFU_DIALOG_SUBTITLE_EN),
        (UiLang::Zh, Msg::CenterTofuDialogSubtitle) => {
            Some("该宿主机尚未在信任列表中，请先比对其 Titan Host 设置页中的指纹是否一致再继续。")
        }
        (UiLang::En, Msg::CenterTofuHostLabel) => Some("Host address"),
        (UiLang::Zh, Msg::CenterTofuHostLabel) => Some("宿主地址"),
        (UiLang::En, Msg::CenterTofuFingerprintLabel) => Some("SPKI fingerprint (sha256)"),
        (UiLang::Zh, Msg::CenterTofuFingerprintLabel) => Some("SPKI 指纹（sha256）"),
        (UiLang::En, Msg::CenterTofuWarning) => Some(CENTER_TOFU_WARNING_EN),
        (UiLang::Zh, Msg::CenterTofuWarning) => {
            Some("信任未知指纹存在中间人风险，请确认指纹与目标宿主机本地显示一致后再点击。")
        }
        (UiLang::En, Msg::CenterTofuConfirm) => Some("Trust and connect"),
        (UiLang::Zh, Msg::CenterTofuConfirm) => Some("信任并连接"),
        _ => None,
    }
}

fn center_settings_mtls(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::CenterSettingsMtlsSection) => Some("mTLS / trusted hosts"),
        (UiLang::Zh, Msg::CenterSettingsMtlsSection) => Some("mTLS / 受信宿主机"),
        (UiLang::En, Msg::CenterSettingsLocalFingerprint) => Some("Local fingerprint"),
        (UiLang::Zh, Msg::CenterSettingsLocalFingerprint) => Some("本机指纹"),
        (UiLang::En, Msg::CenterSettingsTrustedHosts) => Some("Trusted hosts"),
        (UiLang::Zh, Msg::CenterSettingsTrustedHosts) => Some("受信宿主机"),
        (UiLang::En, Msg::CenterSettingsNoTrustedHosts) => {
            Some("No host has been trusted yet (LAN announcements add hosts automatically).")
        }
        (UiLang::Zh, Msg::CenterSettingsNoTrustedHosts) => {
            Some("尚未信任任何宿主机（局域网注册会自动加入受信列表）。")
        }
        _ => None,
    }
}

fn center_vm_window_create_i18n(lang: UiLang, msg: Msg) -> Option<&'static str> {
    center_vm_window_create_i18n_devices(lang, msg)
        .or_else(|| center_vm_window_create_i18n_sync_api(lang, msg))
}

fn center_vm_window_create_i18n_devices(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::CenterWinMgmtDevice) => Some("Device"),
        (UiLang::Zh, Msg::CenterWinMgmtDevice) => Some("设备"),
        (UiLang::En, Msg::CenterWinMgmtAllDevices) => Some("All devices"),
        (UiLang::Zh, Msg::CenterWinMgmtAllDevices) => Some("全部设备"),
        (UiLang::En, Msg::CenterWinMgmtDevicePlaceholder) => Some("Select a device (required)"),
        (UiLang::Zh, Msg::CenterWinMgmtDevicePlaceholder) => Some("请选择设备（必填）"),
        (UiLang::En, Msg::CenterWinMgmtErrNoDevice) => Some("Please select a device."),
        (UiLang::Zh, Msg::CenterWinMgmtErrNoDevice) => Some("请选择设备。"),
        (UiLang::En, Msg::CenterWinMgmtErrNoDevices) => {
            Some("No registered devices. Add a host on the Connect tab first.")
        }
        (UiLang::Zh, Msg::CenterWinMgmtErrNoDevices) => {
            Some("暂无已登记设备，请先在「连接」页添加宿主机。")
        }
        (UiLang::En, Msg::CenterWinMgmtDbErr) => Some("Could not save to the database."),
        (UiLang::Zh, Msg::CenterWinMgmtDbErr) => Some("无法写入数据库。"),
        (UiLang::En, Msg::CenterWinMgmtToastCreated) => {
            Some("VM window saved for the selected host.")
        }
        (UiLang::Zh, Msg::CenterWinMgmtToastCreated) => Some("已为所选宿主机保存虚拟机窗口。"),
        _ => None,
    }
}

fn center_vm_window_create_i18n_sync_api(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::CenterWinMgmtHostSyncErr) => Some(
            "Could not sync this window to the host (offline, protocol mismatch, or rejected).",
        ),
        (UiLang::Zh, Msg::CenterWinMgmtHostSyncErr) => {
            Some("无法将窗口同步到宿主机（离线、协议不匹配或已拒绝）。")
        }
        (UiLang::En, Msg::CenterWinMgmtErrVmDup) => {
            Some("This host already has that VM ID or the same VM directory.")
        }
        (UiLang::Zh, Msg::CenterWinMgmtErrVmDup) => {
            Some("该宿主机已存在相同的虚拟机 ID 或虚拟机目录。")
        }
        (UiLang::En, Msg::CenterVmWindowApiTcpPort) => Some("VM window list TCP port"),
        (UiLang::Zh, Msg::CenterVmWindowApiTcpPort) => Some("虚拟机窗口列表 TCP 端口"),
        (UiLang::En, Msg::CenterVmWindowApiTcpPortHint) => Some(
            "Titan Host uses this port to fetch rows from the center DB; restart Titan Center after changing.",
        ),
        (UiLang::Zh, Msg::CenterVmWindowApiTcpPortHint) => {
            Some("Titan Host 经此端口从本机数据库拉取窗口行；修改后请重启 Titan 中控。")
        }
        _ => None,
    }
}

fn tray(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::TrayShowMainWindow) => Some("Show main window"),
        (UiLang::Zh, Msg::TrayShowMainWindow) => Some("显示主窗口"),
        (UiLang::En, Msg::TrayQuit) => Some("Quit"),
        (UiLang::Zh, Msg::TrayQuit) => Some("退出"),
        _ => None,
    }
}

fn inventory_and_preview(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::WinMgmtReloadDb) => Some("Reload list"),
        (UiLang::Zh, Msg::WinMgmtReloadDb) => Some("重新加载列表"),
        (UiLang::En, Msg::WinMgmtNoWindows) => Some("No registered VM windows yet"),
        (UiLang::Zh, Msg::WinMgmtNoWindows) => Some("暂无已登记的虚拟机窗口"),
        (UiLang::En, Msg::WinMgmtEmptyHint) => Some(
            "Use Create window here and pick a device, or register from Titan Host on each node (LAN UDP sync).",
        ),
        (UiLang::Zh, Msg::WinMgmtEmptyHint) => Some(
            "在此点击「创建窗口」并选择设备即可登记；或在各节点 Titan Host 创建后由局域网（UDP）同步到此处。",
        ),
        (UiLang::En, Msg::ColState) => Some("State"),
        (UiLang::Zh, Msg::ColState) => Some("状态"),
        (UiLang::En, Msg::VmTileHostPrefix) => Some("Host"),
        (UiLang::Zh, Msg::VmTileHostPrefix) => Some("宿主"),
        (UiLang::En, Msg::NoHost) => Some("no-host"),
        (UiLang::Zh, Msg::NoHost) => Some("未选主机"),
        _ => None,
    }
}

fn monitor_counts(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::MonitorCardDevices) => Some("Devices"),
        (UiLang::Zh, Msg::MonitorCardDevices) => Some("设备数量"),
        (UiLang::En, Msg::MonitorCardWindows) => Some("Windows"),
        (UiLang::Zh, Msg::MonitorCardWindows) => Some("窗口数"),
        (UiLang::En, Msg::MonitorStatTotal) => Some("Total"),
        (UiLang::Zh, Msg::MonitorStatTotal) => Some("总数"),
        (UiLang::En, Msg::MonitorStatOnline) => Some("Online"),
        (UiLang::Zh, Msg::MonitorStatOnline) => Some("在线"),
        (UiLang::En, Msg::MonitorStatNotBooted) => Some("Not booted"),
        (UiLang::Zh, Msg::MonitorStatNotBooted) => Some("未开机"),
        (UiLang::En, Msg::MonitorStatOffline) => Some("Offline"),
        (UiLang::Zh, Msg::MonitorStatOffline) => Some("离线"),
        _ => None,
    }
}

fn monitor_hints_and_actions(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::MonitorDevicesScopeHint) => Some(
            "Online = last successful connect (Hello/Ping) while this device row was selected.",
        ),
        (UiLang::Zh, Msg::MonitorDevicesScopeHint) => {
            Some("在线=该行设备为当前会话目标且最近一次连接成功（Hello/Ping）。")
        }
        (UiLang::En, Msg::MonitorWindowsScopeHint) => {
            Some("Running vs other states from the last List VMs on the selected device.")
        }
        (UiLang::Zh, Msg::MonitorWindowsScopeHint) => {
            Some("在线=运行中；统计来自当前选中设备最近一次「列出虚拟机」。")
        }
        _ => None,
    }
}
