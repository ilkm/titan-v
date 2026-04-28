//! Titan Host egui panel strings (EN/ZH).

use crate::{Msg, UiLang};

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    translate_hp_chrome(lang, msg)
        .or_else(|| translate_hp_service_listen(lang, msg))
        .or_else(|| translate_hp_batch_a(lang, msg))
        .or_else(|| translate_hp_batch_b(lang, msg))
}

fn translate_hp_chrome(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpWinTitle) => Some("Titan Host"),
        (UiLang::Zh, Msg::HpWinTitle) => Some("Titan 客户端"),
        (UiLang::En, Msg::HpTabService) => Some("Service"),
        (UiLang::Zh, Msg::HpTabService) => Some("服务"),
        (UiLang::En, Msg::HpTabBatch) => Some("Batch create"),
        (UiLang::Zh, Msg::HpTabBatch) => Some("批量创建"),
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

fn translate_hp_batch_a(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpBatchTimeout) => Some("Timeout per VM (seconds)"),
        (UiLang::Zh, Msg::HpBatchTimeout) => Some("每台 VM 超时 (秒)"),
        (UiLang::En, Msg::HpBatchFailFast) => Some("Stop on first error (fail-fast)"),
        (UiLang::Zh, Msg::HpBatchFailFast) => Some("遇错即停 (fail-fast)"),
        (UiLang::En, Msg::HpHeadingVmPlans) => Some("Explicit VMs (VmProvisionPlan)"),
        (UiLang::Zh, Msg::HpHeadingVmPlans) => Some("显式 VM (VmProvisionPlan)"),
        (UiLang::En, Msg::HpHeadingVmGroups) => Some("VM group template (vm_group)"),
        (UiLang::Zh, Msg::HpHeadingVmGroups) => Some("VM 组模板 (vm_group)"),
        (UiLang::En, Msg::HpAddExplicitVm) => Some("Add explicit VM"),
        (UiLang::Zh, Msg::HpAddExplicitVm) => Some("添加显式 VM"),
        (UiLang::En, Msg::HpAddVmGroup) => Some("Add VM group"),
        (UiLang::Zh, Msg::HpAddVmGroup) => Some("添加 VM 组"),
        (UiLang::En, Msg::HpName) => Some("Name"),
        (UiLang::Zh, Msg::HpName) => Some("名称"),
        (UiLang::En, Msg::HpDelete) => Some("Delete"),
        (UiLang::Zh, Msg::HpDelete) => Some("删除"),
        (UiLang::En, Msg::HpParentVhdx) => Some("Parent VHDX"),
        (UiLang::Zh, Msg::HpParentVhdx) => Some("父 VHDX"),
        (UiLang::En, Msg::HpDiffDir) => Some("Diff directory"),
        (UiLang::Zh, Msg::HpDiffDir) => Some("差分目录"),
        (UiLang::En, Msg::HpMemBytes) => Some("Memory (bytes)"),
        (UiLang::Zh, Msg::HpMemBytes) => Some("内存 (字节)"),
        (UiLang::En, Msg::HpGen) => Some("Generation"),
        (UiLang::Zh, Msg::HpGen) => Some("代数"),
        _ => None,
    }
}

fn translate_hp_batch_b(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::HpSwitch) => Some("Switch (empty = none)"),
        (UiLang::Zh, Msg::HpSwitch) => Some("交换机 (空=无)"),
        (UiLang::En, Msg::HpAutoStartAfter) => Some("Auto-start after create"),
        (UiLang::Zh, Msg::HpAutoStartAfter) => Some("创建后自动启动"),
        (UiLang::En, Msg::HpGpuPath) => Some("GPU instance path (optional)"),
        (UiLang::Zh, Msg::HpGpuPath) => Some("GPU 实例路径 (可选)"),
        (UiLang::En, Msg::HpDynMac) => Some("Dynamic MAC"),
        (UiLang::Zh, Msg::HpDynMac) => Some("动态 MAC"),
        (UiLang::En, Msg::HpNoCkpt) => Some("Disable checkpoints"),
        (UiLang::Zh, Msg::HpNoCkpt) => Some("禁用检查点"),
        (UiLang::En, Msg::HpCpuCount) => Some("CPU count (0=default)"),
        (UiLang::Zh, Msg::HpCpuCount) => Some("CPU 数 (0=默认)"),
        (UiLang::En, Msg::HpPrefix) => Some("Prefix"),
        (UiLang::Zh, Msg::HpPrefix) => Some("前缀"),
        (UiLang::En, Msg::HpCount) => Some("Count"),
        (UiLang::Zh, Msg::HpCount) => Some("数量"),
        (UiLang::En, Msg::HpDelGroup) => Some("Remove group"),
        (UiLang::Zh, Msg::HpDelGroup) => Some("删除组"),
        (UiLang::En, Msg::HpDryRun) => Some("Dry-run"),
        (UiLang::Zh, Msg::HpDryRun) => Some("预检 (dry-run)"),
        (UiLang::En, Msg::HpProvision) => Some("Start create"),
        (UiLang::Zh, Msg::HpProvision) => Some("开始创建"),
        (UiLang::En, Msg::HpLog) => Some("Log"),
        (UiLang::Zh, Msg::HpLog) => Some("日志"),
        _ => None,
    }
}
