use super::msg::Msg;
use super::UiLang;

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    host_collect_block(lang, msg)
        .or_else(|| device_empty_and_preview(lang, msg))
        .or_else(|| device_metrics(lang, msg))
        .or_else(|| add_host_dialog(lang, msg))
        .or_else(|| add_host_verify_and_toolbar(lang, msg))
}

fn host_collect_block(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HostCollectTitle) => Some("LAN host registration (scale-out)"),
        (UiLang::Zh, Msg::HostCollectTitle) => Some("局域网宿主注册（批量）"),
        (UiLang::En, Msg::HostCollectBlurb) => {
            Some("Broadcasts a short poll on UDP. Each `titan-host serve` on the LAN replies with its control-plane TCP address — no per-host Connect click. Uses the same bind-IPv4 list as guest discovery when set.")
        }
        (UiLang::Zh, Msg::HostCollectBlurb) => {
            Some("在局域网周期性广播唤请包（UDP）。各台 `titan-host serve` 收到后自动向中控上报控制面 TCP 地址，无需在每台机器上单独操作中控。若上方指定了绑定 IPv4，则与本页「来宾发现」共用该列表。")
        }
        (UiLang::En, Msg::HostCollectCheckbox) => Some("Broadcast registration poll"),
        (UiLang::Zh, Msg::HostCollectCheckbox) => Some("广播唤请宿主注册"),
        (UiLang::En, Msg::HostCollectIntervalLabel) => Some("Poll interval (seconds):"),
        (UiLang::Zh, Msg::HostCollectIntervalLabel) => Some("唤请间隔（秒）："),
        (UiLang::En, Msg::HostCollectPollPortLabel) => Some("Poll UDP port (destination):"),
        (UiLang::Zh, Msg::HostCollectPollPortLabel) => Some("唤请 UDP 端口（目的）："),
        (UiLang::En, Msg::HostCollectRegisterPortLabel) => {
            Some("Host reply UDP port (center listens):")
        }
        (UiLang::Zh, Msg::HostCollectRegisterPortLabel) => {
            Some("宿主应答 UDP 端口（中控监听）：")
        }
        _ => None,
    }
}

fn device_empty_and_preview(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::NoDataShort) => Some("No data"),
        (UiLang::Zh, Msg::NoDataShort) => Some("暂无数据"),
        (UiLang::En, Msg::DeviceMgmtNoRegistered) => Some("No registered devices yet"),
        (UiLang::Zh, Msg::DeviceMgmtNoRegistered) => Some("暂无注册设备～"),
        (UiLang::En, Msg::DeviceMgmtEmptyHint) => {
            Some("No hosts yet. Use **Settings** → LAN discovery / LAN host registration; devices also merge here when they announce on UDP.")
        }
        (UiLang::Zh, Msg::DeviceMgmtEmptyHint) => {
            Some("暂无设备。请打开左侧「设置」使用局域网发现或局域网宿主注册；宿主 UDP 宣告后也会出现在此列表。")
        }
        (UiLang::En, Msg::DeviceMgmtDesktopPreviewNote) => {
            Some("Desktop preview — live capture not connected yet")
        }
        (UiLang::Zh, Msg::DeviceMgmtDesktopPreviewNote) => {
            Some("桌面预览 — 尚未接入实时画面")
        }
        _ => None,
    }
}

fn device_metrics(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::DeviceMgmtResCpu) => Some("CPU"),
        (UiLang::Zh, Msg::DeviceMgmtResCpu) => Some("CPU"),
        (UiLang::En, Msg::DeviceMgmtResMem) => Some("Mem"),
        (UiLang::Zh, Msg::DeviceMgmtResMem) => Some("内存"),
        (UiLang::En, Msg::DeviceMgmtResNet) => Some("Net"),
        (UiLang::Zh, Msg::DeviceMgmtResNet) => Some("网速"),
        (UiLang::En, Msg::DeviceMgmtResDiskIo) => Some("Disk"),
        (UiLang::Zh, Msg::DeviceMgmtResDiskIo) => Some("磁盘"),
        (UiLang::En, Msg::DeviceMgmtRemarkTitle) => Some("Note"),
        (UiLang::Zh, Msg::DeviceMgmtRemarkTitle) => Some("备注"),
        (UiLang::En, Msg::DeviceMgmtRemarkDblclkHint) => Some("Double-click to add a note…"),
        (UiLang::Zh, Msg::DeviceMgmtRemarkDblclkHint) => Some("双击添加备注…"),
        _ => None,
    }
}

fn add_host_dialog(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::BtnAddHost) => Some("+ Add host"),
        (UiLang::Zh, Msg::BtnAddHost) => Some("+ 添加主机"),
        (UiLang::En, Msg::AddHostDialogTitle) => Some("Add host"),
        (UiLang::Zh, Msg::AddHostDialogTitle) => Some("添加主机"),
        (UiLang::En, Msg::AddHostDialogSubtitle) => {
            Some("Control-plane address on the machine running `titan-host serve`. The host must be online — we send Hello before adding.")
        }
        (UiLang::Zh, Msg::AddHostDialogSubtitle) => {
            Some("填写运行 `titan-host serve` 的机器上的控制面 TCP 地址。添加前会检测在线（发送 Hello）。")
        }
        (UiLang::En, Msg::AddHostIpLabel) => Some("IPv4 address"),
        (UiLang::Zh, Msg::AddHostIpLabel) => Some("IPv4 地址"),
        (UiLang::En, Msg::AddHostPortLabel) => Some("TCP port"),
        (UiLang::Zh, Msg::AddHostPortLabel) => Some("TCP 端口"),
        (UiLang::En, Msg::AddHostConfirm) => Some("Add"),
        (UiLang::Zh, Msg::AddHostConfirm) => Some("添加"),
        _ => None,
    }
}

fn add_host_verify_and_toolbar(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::AddHostInvalidHint) => {
            Some("Enter a valid IPv4 and a port from 1 to 65535.")
        }
        (UiLang::Zh, Msg::AddHostInvalidHint) => {
            Some("请输入有效的 IPv4 与 1–65535 范围内的端口。")
        }
        (UiLang::En, Msg::AddHostVerifying) => Some("Checking…"),
        (UiLang::Zh, Msg::AddHostVerifying) => Some("正在检测…"),
        (UiLang::En, Msg::AddHostOfflineToast) => Some("Device is offline."),
        (UiLang::Zh, Msg::AddHostOfflineToast) => Some("设备不在线～"),
        (UiLang::En, Msg::AddHostSavedLog) => Some("Host added (online check OK)."),
        (UiLang::Zh, Msg::AddHostSavedLog) => Some("已添加主机（在线检测通过）。"),
        (UiLang::En, Msg::BtnRemoveSelected) => Some("Remove selected"),
        (UiLang::Zh, Msg::BtnRemoveSelected) => Some("删除选中"),
        (UiLang::En, Msg::BtnHostHello) => Some("Hello"),
        (UiLang::Zh, Msg::BtnHostHello) => Some("Hello"),
        (UiLang::En, Msg::BtnHostTelemetry) => Some("Telemetry"),
        (UiLang::Zh, Msg::BtnHostTelemetry) => Some("遥测"),
        _ => None,
    }
}
