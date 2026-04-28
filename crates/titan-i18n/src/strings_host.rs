//! Titan Host egui panel strings (EN/ZH).

use crate::{Msg, UiLang};

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_chrome(lang, msg)
        .or_else(|| translate_hp_service_listen(lang, msg))
        .or_else(|| translate_hp_settings_sections(lang, msg))
}

fn translate_hp_chrome(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinTitle) => Some("Titan Host"),
        (UiLang::Zh, Msg::HpWinTitle) => Some("Titan 客户端"),
        (UiLang::En, Msg::HpTabSettings) => Some("Settings"),
        (UiLang::Zh, Msg::HpTabSettings) => Some("设置"),
        (UiLang::En, Msg::HpTabWindowMgmt) => Some("Window"),
        (UiLang::Zh, Msg::HpTabWindowMgmt) => Some("窗口管理"),
        (UiLang::En, Msg::HpLangLabel) => Some("Language"),
        (UiLang::Zh, Msg::HpLangLabel) => Some("语言"),
        _ => None,
    }
}

fn translate_hp_service_listen(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpListen) => Some("Listen address"),
        (UiLang::Zh, Msg::HpListen) => Some("监听地址"),
        (UiLang::En, Msg::HpAnnounce) => Some("LAN registration / replies"),
        (UiLang::Zh, Msg::HpAnnounce) => Some("启用 LAN 注册 / 应答"),
        (UiLang::En, Msg::HpPollPort) => Some("Poll UDP port"),
        (UiLang::Zh, Msg::HpPollPort) => Some("轮询 UDP 端口"),
        (UiLang::En, Msg::HpRegPort) => Some("Register UDP port"),
        (UiLang::Zh, Msg::HpRegPort) => Some("注册 UDP 端口"),
        (UiLang::En, Msg::HpPeriodic) => Some("Periodic announce (s, 0=off)"),
        (UiLang::Zh, Msg::HpPeriodic) => Some("周期广播间隔 (秒，0=关闭)"),
        (UiLang::En, Msg::HpPublicAddr) => Some("Public / display address override"),
        (UiLang::Zh, Msg::HpPublicAddr) => Some("公网/展示地址覆盖"),
        (UiLang::En, Msg::HpLabelOverride) => Some("Host label override"),
        (UiLang::Zh, Msg::HpLabelOverride) => Some("主机标签覆盖"),
        (UiLang::En, Msg::HpSaveRestart) => Some("Save & restart control plane"),
        (UiLang::Zh, Msg::HpSaveRestart) => Some("保存并重启控制面"),
        _ => None,
    }
}

fn translate_hp_settings_sections(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpSectionControlPlane) => Some("Control plane"),
        (UiLang::Zh, Msg::HpSectionControlPlane) => Some("控制面"),
        (UiLang::En, Msg::HpSectionLanAnnounce) => Some("LAN registration"),
        (UiLang::Zh, Msg::HpSectionLanAnnounce) => Some("局域网注册"),
        (UiLang::En, Msg::HpSectionIdentity) => Some("Host identity"),
        (UiLang::Zh, Msg::HpSectionIdentity) => Some("主机标识"),
        _ => None,
    }
}
