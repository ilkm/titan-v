use crate::UiLang;
use crate::msg::Msg;

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    inventory_and_preview(lang, msg)
        .or_else(|| monitor_counts(lang, msg))
        .or_else(|| monitor_hints_and_actions(lang, msg))
        .or_else(|| slots(lang, msg))
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
        (UiLang::En, Msg::VmInventoryTitle) => Some("VM inventory"),
        (UiLang::Zh, Msg::VmInventoryTitle) => Some("虚拟机清单"),
        (UiLang::En, Msg::ColState) => Some("State"),
        (UiLang::Zh, Msg::ColState) => Some("状态"),
        (UiLang::En, Msg::VmTileHostPrefix) => Some("Host"),
        (UiLang::Zh, Msg::VmTileHostPrefix) => Some("宿主"),
        (UiLang::En, Msg::WindowPreviewTitle) => Some("VM preview"),
        (UiLang::Zh, Msg::WindowPreviewTitle) => Some("画面预览"),
        (UiLang::En, Msg::WindowPreviewHint) => {
            Some("Placeholder for per-window video (host · VM · slot 1–40 wiring comes later).")
        }
        (UiLang::Zh, Msg::WindowPreviewHint) => {
            Some("视频预览占位。后续按「宿主机名 · 虚拟机 · 窗口编号 1–40」接入推流。")
        }
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

fn slots(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::SlotGridTitle) => Some("Slot grid (virtualized)"),
        (UiLang::Zh, Msg::SlotGridTitle) => Some("槽位网格（虚拟化示例）"),
        (UiLang::En, Msg::SlotEmpty) => Some("empty"),
        (UiLang::Zh, Msg::SlotEmpty) => Some("空"),
        (UiLang::En, Msg::NoHost) => Some("no-host"),
        (UiLang::Zh, Msg::NoHost) => Some("未选主机"),
        _ => None,
    }
}
