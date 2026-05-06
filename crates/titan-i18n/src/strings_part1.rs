use crate::UiLang;
use crate::msg::Msg;

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    brand_settings_lang(lang, msg)
        .or_else(|| nav_all(lang, msg))
        .or_else(|| discovery_top(lang, msg))
        .or_else(|| discovery_bind_ports(lang, msg))
}

fn brand_settings_lang(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::BrandTitle) => Some("Titan Center"),
        (UiLang::Zh, Msg::BrandTitle) => Some("Titan 中控端"),
        (UiLang::En, Msg::SettingsTooltip) => Some("Settings"),
        (UiLang::Zh, Msg::SettingsTooltip) => Some("设置"),
        (UiLang::En, Msg::SettingsLangWindowTitle) => Some("Language settings"),
        (UiLang::Zh, Msg::SettingsLangWindowTitle) => Some("语言设置"),
        (UiLang::En, Msg::LangRadioEn) => Some("English"),
        (UiLang::Zh, Msg::LangRadioEn) => Some("English"),
        (UiLang::En, Msg::LangRadioZh) => Some("中文"),
        (UiLang::Zh, Msg::LangRadioZh) => Some("中文"),
        _ => None,
    }
}

fn nav_all(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::NavConnect) => Some("Devices"),
        (UiLang::Zh, Msg::NavConnect) => Some("设备管理"),
        (UiLang::En, Msg::NavSettings) => Some("Settings"),
        (UiLang::Zh, Msg::NavSettings) => Some("设置"),
        (UiLang::En, Msg::NavHostsVms) => Some("Windows"),
        (UiLang::Zh, Msg::NavHostsVms) => Some("窗口管理"),
        (UiLang::En, Msg::NavMonitor) => Some("Usage"),
        (UiLang::Zh, Msg::NavMonitor) => Some("资源监控"),
        _ => None,
    }
}

fn discovery_top(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::DiscoveryTitle) => Some("LAN discovery"),
        (UiLang::Zh, Msg::DiscoveryTitle) => Some("局域网发现"),
        (UiLang::En, Msg::DiscoveryUdpBlurb) => Some(
            "UDP broadcast of `DiscoveryBeacon` so in-guest automation can learn this control address.",
        ),
        (UiLang::Zh, Msg::DiscoveryUdpBlurb) => {
            Some("通过 UDP 广播 `DiscoveryBeacon`，便于来宾内自动化获知本控制地址。")
        }
        (UiLang::En, Msg::DiscoveryCheckbox) => Some("Broadcast discovery on LAN"),
        (UiLang::Zh, Msg::DiscoveryCheckbox) => Some("在局域网广播发现"),
        _ => None,
    }
}

fn discovery_bind_ports(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::DiscoveryBindBlurb) => Some(
            "Optional: pick one or more local IPv4 addresses to send subnet broadcasts from (multi-homed). Leave none selected to use the OS default route (255.255.255.255).",
        ),
        (UiLang::Zh, Msg::DiscoveryBindBlurb) => Some(
            "可选：选择一个或多个本机 IPv4，从对应网卡向子网广播；不选则走系统默认路由（255.255.255.255）。",
        ),
        (UiLang::En, Msg::DiscoveryBindQuickAdd) => Some("Add interface IPv4…"),
        (UiLang::Zh, Msg::DiscoveryBindQuickAdd) => Some("从列表添加 IPv4…"),
        (UiLang::En, Msg::DiscoveryBindScrollHint) => Some("Multi-select (non-loopback IPv4):"),
        (UiLang::Zh, Msg::DiscoveryBindScrollHint) => Some("多选广播源（不含回环）："),
        (UiLang::En, Msg::DiscoveryRefreshIfaces) => Some("Refresh interfaces"),
        (UiLang::Zh, Msg::DiscoveryRefreshIfaces) => Some("刷新网卡列表"),
        (UiLang::En, Msg::DiscoveryClearBindIps) => Some("Clear selection"),
        (UiLang::Zh, Msg::DiscoveryClearBindIps) => Some("清空已选"),
        (UiLang::En, Msg::DiscoverySelectAllBindIps) => Some("Select all"),
        (UiLang::Zh, Msg::DiscoverySelectAllBindIps) => Some("全选"),
        (UiLang::En, Msg::DiscoveryNoIpv4Ifaces) => Some("No non-loopback IPv4 interfaces found."),
        (UiLang::Zh, Msg::DiscoveryNoIpv4Ifaces) => Some("未发现可用的非回环 IPv4 网卡。"),
        (UiLang::En, Msg::IntervalLabel) => Some("Interval (s):"),
        (UiLang::Zh, Msg::IntervalLabel) => Some("间隔（秒）："),
        (UiLang::En, Msg::UdpPortLabel) => Some("UDP port:"),
        (UiLang::Zh, Msg::UdpPortLabel) => Some("UDP 端口："),
        _ => None,
    }
}
