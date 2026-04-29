use crate::UiLang;
use crate::msg::Msg;

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    inventory_and_preview(lang, msg)
        .or_else(|| monitor_counts(lang, msg))
        .or_else(|| monitor_hints_and_actions(lang, msg))
        .or_else(|| tray(lang, msg))
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
        (UiLang::En, Msg::WinMgmtEmptyHint) => {
            Some("Create a window from Titan Host on each machine; rows sync here over LAN (UDP).")
        }
        (UiLang::Zh, Msg::WinMgmtEmptyHint) => {
            Some("在各节点 Titan Host 的「窗口管理」中创建窗口后，将通过局域网（UDP）同步到此处。")
        }
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
