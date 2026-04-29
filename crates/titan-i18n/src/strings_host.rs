//! Titan Host egui panel strings (EN/ZH).

use crate::{Msg, UiLang};

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_chrome(lang, msg)
        .or_else(|| translate_hp_service_listen(lang, msg))
        .or_else(|| translate_hp_settings_sections(lang, msg))
        .or_else(|| translate_hp_window_mgmt(lang, msg))
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

fn translate_hp_window_mgmt(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_window_mgmt_fields(lang, msg)
        .or_else(|| translate_hp_window_mgmt_status(lang, msg))
}

fn translate_hp_window_mgmt_fields(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinMgmtCreateBtn) => Some("Create window"),
        (UiLang::Zh, Msg::HpWinMgmtCreateBtn) => Some("创建窗口"),
        (UiLang::En, Msg::HpWinMgmtDialogTitle) => Some("New window"),
        (UiLang::Zh, Msg::HpWinMgmtDialogTitle) => Some("新建窗口"),
        (UiLang::En, Msg::HpWinMgmtCpu) => Some("CPU count"),
        (UiLang::Zh, Msg::HpWinMgmtCpu) => Some("CPU 数量"),
        (UiLang::En, Msg::HpWinMgmtMem) => Some("Memory (MiB, 1024 = 1 GiB)"),
        (UiLang::Zh, Msg::HpWinMgmtMem) => Some("内存 (MiB，1024 = 1 GiB)"),
        (UiLang::En, Msg::HpWinMgmtDisk) => Some("Disk (MiB, 1024 = 1 GiB)"),
        (UiLang::Zh, Msg::HpWinMgmtDisk) => Some("磁盘 (MiB，1024 = 1 GiB)"),
        (UiLang::En, Msg::HpWinMgmtVmDir) => Some("VM directory"),
        (UiLang::Zh, Msg::HpWinMgmtVmDir) => Some("虚拟机目录"),
        (UiLang::En, Msg::HpWinMgmtVmDirHint) => Some("~/titan/vm/001 (auto-increment)"),
        (UiLang::Zh, Msg::HpWinMgmtVmDirHint) => Some("~/titan/vm/001（编号自动递增）"),
        (UiLang::En, Msg::HpWinMgmtConfirm) => Some("Create"),
        (UiLang::Zh, Msg::HpWinMgmtConfirm) => Some("创建"),
        _ => None,
    }
}

fn translate_hp_window_mgmt_status(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinMgmtErrDir) => Some("VM directory is required."),
        (UiLang::Zh, Msg::HpWinMgmtErrDir) => Some("请填写虚拟机目录。"),
        (UiLang::En, Msg::HpWinMgmtSavedNotified) => {
            Some("Saved locally and notified Titan Center (SQLite).")
        }
        (UiLang::Zh, Msg::HpWinMgmtSavedNotified) => Some("已保存并已通知中控（写入数据库）。"),
        (UiLang::En, Msg::HpWinMgmtSaveErr) => Some("Could not save the window list locally."),
        (UiLang::Zh, Msg::HpWinMgmtSaveErr) => Some("无法将窗口列表保存到本地。"),
        _ => None,
    }
}
