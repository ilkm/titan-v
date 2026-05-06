//! Titan Host egui panel strings (EN/ZH).

use crate::{Msg, UiLang};

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_chrome(lang, msg)
        .or_else(|| translate_hp_service_listen(lang, msg))
        .or_else(|| translate_hp_settings_sections(lang, msg))
        .or_else(|| translate_hp_pairing(lang, msg))
        .or_else(|| translate_hp_window_mgmt(lang, msg))
}

fn translate_hp_pairing(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpSectionMtlsPairing) => Some("mTLS pairing"),
        (UiLang::Zh, Msg::HpSectionMtlsPairing) => Some("mTLS 配对"),
        (UiLang::En, Msg::HpQuicFingerprintLabel) => Some("Local fingerprint"),
        (UiLang::Zh, Msg::HpQuicFingerprintLabel) => Some("本机指纹"),
        (UiLang::En, Msg::HpQuicPairingOpenBtn) => Some("Open pairing window (5 min)"),
        (UiLang::Zh, Msg::HpQuicPairingOpenBtn) => Some("开启配对窗口（5 分钟）"),
        (UiLang::En, Msg::HpQuicPairingClose) => Some("Close pairing window"),
        (UiLang::Zh, Msg::HpQuicPairingClose) => Some("关闭配对窗口"),
        (UiLang::En, Msg::HpQuicTrustedCentersHeader) => Some("Trusted centers"),
        (UiLang::Zh, Msg::HpQuicTrustedCentersHeader) => Some("已信任的中控"),
        (UiLang::En, Msg::HpQuicNoTrustedCenters) => Some("No center has paired yet."),
        (UiLang::Zh, Msg::HpQuicNoTrustedCenters) => Some("尚未有中控完成配对。"),
        _ => None,
    }
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
        (UiLang::En, Msg::HpLanBindIface) => Some("LAN NIC"),
        (UiLang::Zh, Msg::HpLanBindIface) => Some("局域网网卡"),
        (UiLang::En, Msg::HpLanBindIfaceNone) => Some("No physical LAN IPv4"),
        (UiLang::Zh, Msg::HpLanBindIfaceNone) => Some("无可用物理局域网 IPv4"),
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
        (UiLang::En, Msg::HpSectionVmStorage) => Some("VM storage"),
        (UiLang::Zh, Msg::HpSectionVmStorage) => Some("虚拟机存储"),
        (UiLang::En, Msg::HpVmRootDir) => Some("VM root directory"),
        (UiLang::Zh, Msg::HpVmRootDir) => Some("虚拟机根目录"),
        (UiLang::En, Msg::HpVmRootDirHint) => Some("Leave empty to use ~/titan/vm"),
        (UiLang::Zh, Msg::HpVmRootDirHint) => Some("留空则使用 ~/titan/vm"),
        (UiLang::En, Msg::HpCenterVmWindowApiAddr) => Some("Titan Center VM list API (host:port)"),
        (UiLang::Zh, Msg::HpCenterVmWindowApiAddr) => Some("中控虚拟机窗口接口 (主机:端口)"),
        (UiLang::En, Msg::HpCenterVmWindowApiAddrHint) => {
            Some("TCP on the port set in Titan Center settings (default 7793).")
        }
        (UiLang::Zh, Msg::HpCenterVmWindowApiAddrHint) => {
            Some("与 Titan 中控设置中的 TCP 端口一致（默认 7793）。")
        }
        _ => None,
    }
}

fn translate_hp_window_mgmt(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_window_mgmt_fields(lang, msg)
        .or_else(|| translate_hp_window_mgmt_status(lang, msg))
}

fn translate_hp_window_mgmt_fields(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_window_mgmt_fields_modal(lang, msg)
        .or_else(|| translate_hp_window_mgmt_fields_ids(lang, msg))
}

fn translate_hp_window_mgmt_fields_modal(lang: UiLang, msg: Msg) -> Option<&'static str> {
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
        _ => None,
    }
}

fn translate_hp_window_mgmt_fields_ids(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinMgmtVmDir) => Some("VM directory"),
        (UiLang::Zh, Msg::HpWinMgmtVmDir) => Some("虚拟机目录"),
        (UiLang::En, Msg::HpWinMgmtVmDirHint) => Some("~/titan/vm/001 (auto-increment)"),
        (UiLang::Zh, Msg::HpWinMgmtVmDirHint) => Some("~/titan/vm/001（编号自动递增）"),
        (UiLang::En, Msg::HpWinMgmtVmId) => Some("VM ID"),
        (UiLang::Zh, Msg::HpWinMgmtVmId) => Some("虚拟机 ID"),
        (UiLang::En, Msg::HpWinMgmtVmIdHint) => Some("100–999999999"),
        (UiLang::Zh, Msg::HpWinMgmtVmIdHint) => Some("100–999999999"),
        (UiLang::En, Msg::HpWinMgmtPullCenter) => Some("Sync from Center"),
        (UiLang::Zh, Msg::HpWinMgmtPullCenter) => Some("从中控同步"),
        (UiLang::En, Msg::HpCenterVmApiMissingAddr) => {
            Some("Set Titan Center VM list API (host:port) in Settings first.")
        }
        (UiLang::Zh, Msg::HpCenterVmApiMissingAddr) => {
            Some("请先在设置中填写中控虚拟机窗口接口 (主机:端口)。")
        }
        (UiLang::En, Msg::HpWinMgmtConfirm) => Some("Create"),
        (UiLang::Zh, Msg::HpWinMgmtConfirm) => Some("创建"),
        _ => None,
    }
}

fn translate_hp_window_mgmt_status_ok_err(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinMgmtErrDir) => Some("VM directory is required."),
        (UiLang::Zh, Msg::HpWinMgmtErrDir) => Some("请填写虚拟机目录。"),
        (UiLang::En, Msg::HpWinMgmtSavedNotified) => {
            Some("Titan Center saved this window and the local list has been updated.")
        }
        (UiLang::Zh, Msg::HpWinMgmtSavedNotified) => {
            Some("中控已保存该虚拟机窗口，本地列表已更新。")
        }
        (UiLang::En, Msg::HpWinMgmtSaveErr) => {
            Some("Could not submit the VM window to Titan Center (busy or misconfigured).")
        }
        (UiLang::Zh, Msg::HpWinMgmtSaveErr) => {
            Some("无法向 Titan 中控提交虚拟机窗口（忙或配置异常）。")
        }
        _ => None,
    }
}

fn translate_hp_window_mgmt_status_validation(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinMgmtErrVmId) => Some("VM ID must be between 100 and 999999999."),
        (UiLang::Zh, Msg::HpWinMgmtErrVmId) => Some("虚拟机 ID 须在 100–999999999 之间。"),
        (UiLang::En, Msg::HpWinMgmtErrVmRoot) => {
            Some("Set a VM root directory in Settings (or ensure a home directory exists).")
        }
        (UiLang::Zh, Msg::HpWinMgmtErrVmRoot) => {
            Some("请在设置中填写虚拟机根目录（或确保存在用户主目录）。")
        }
        (UiLang::En, Msg::HpWinMgmtErrVmIdDup) => {
            Some("This VM ID is already in use (matches an existing window path).")
        }
        (UiLang::Zh, Msg::HpWinMgmtErrVmIdDup) => {
            Some("该虚拟机 ID 已存在（与已有窗口路径重复）。")
        }
        _ => None,
    }
}

fn translate_hp_window_mgmt_status(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_window_mgmt_status_ok_err(lang, msg)
        .or_else(|| translate_hp_window_mgmt_status_validation(lang, msg))
}
