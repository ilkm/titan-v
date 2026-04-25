use super::msg::Msg;
use super::UiLang;

pub(super) fn translate(lang: UiLang, msg: Msg) -> Option<&'static str> {
    spoof_head(lang, msg)
        .or_else(|| spoof_controls(lang, msg))
        .or_else(|| spoof_confirm(lang, msg))
        .or_else(|| danger_strings(lang, msg))
}

fn spoof_head(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::SpoofCardTitle) => Some("Host spoof (control plane)"),
        (UiLang::Zh, Msg::SpoofCardTitle) => Some("宿主伪装（控制面）"),
        (UiLang::En, Msg::SpoofBlurb) => {
            Some("Host session must be ready first. Preview = dry-run; Apply runs Hyper-V PowerShell on the host (EULA / law is your responsibility).")
        }
        (UiLang::Zh, Msg::SpoofBlurb) => {
            Some("请先等待宿主会话就绪。预览为 dry-run；应用会在宿主执行 Hyper-V PowerShell（EULA/法律自负）。")
        }
        (UiLang::En, Msg::TargetVmLabel) => Some("Target VM"),
        (UiLang::Zh, Msg::TargetVmLabel) => Some("目标虚拟机"),
        (UiLang::En, Msg::HintTargetVm) => Some("Hyper-V VM name"),
        (UiLang::Zh, Msg::HintTargetVm) => Some("Hyper-V 虚拟机名"),
        _ => None,
    }
}

fn spoof_controls(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::ChkDynamicMac) => {
            Some("Dynamic MAC (Set-VMNetworkAdapter -DynamicMacAddress On)")
        }
        (UiLang::Zh, Msg::ChkDynamicMac) => {
            Some("动态 MAC（Set-VMNetworkAdapter -DynamicMacAddress On）")
        }
        (UiLang::En, Msg::ChkDisableCkpt) => {
            Some("Disable checkpoints (Set-VM -CheckpointType Disabled)")
        }
        (UiLang::Zh, Msg::ChkDisableCkpt) => Some("禁用检查点（Set-VM -CheckpointType Disabled）"),
        (UiLang::En, Msg::BtnPreviewDryRun) => Some("Preview (dry-run)"),
        (UiLang::Zh, Msg::BtnPreviewDryRun) => Some("预览（dry-run）"),
        (UiLang::En, Msg::BtnApplyEllipsis) => Some("Apply…"),
        (UiLang::Zh, Msg::BtnApplyEllipsis) => Some("应用…"),
        _ => None,
    }
}

fn spoof_confirm(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::WinConfirmSpoofTitle) => Some("Confirm spoof apply"),
        (UiLang::Zh, Msg::WinConfirmSpoofTitle) => Some("确认应用伪装"),
        (UiLang::En, Msg::WinConfirmSpoofBody) => {
            Some("This sends ApplySpoofProfile with dry_run=false to the connected host.")
        }
        (UiLang::Zh, Msg::WinConfirmSpoofBody) => {
            Some("将向已连接的宿主发送 ApplySpoofProfile，且 dry_run=false。")
        }
        (UiLang::En, Msg::BtnCancel) => Some("Cancel"),
        (UiLang::Zh, Msg::BtnCancel) => Some("取消"),
        (UiLang::En, Msg::BtnConfirmApply) => Some("Confirm apply"),
        (UiLang::Zh, Msg::BtnConfirmApply) => Some("确认应用"),
        _ => None,
    }
}

fn danger_strings_card_and_actions(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::DangerCardTitle) => Some("Bulk power"),
        (UiLang::Zh, Msg::DangerCardTitle) => Some("批量电源"),
        (UiLang::En, Msg::DangerBlurb) => Some(
            "Comma-separated VM names → StartVmGroup / StopVmGroup on the current control address.",
        ),
        (UiLang::Zh, Msg::DangerBlurb) => {
            Some("逗号分隔的虚拟机名 → 对当前控制地址执行 StartVmGroup / StopVmGroup。")
        }
        (UiLang::En, Msg::HintBulkVms) => Some("vm-01, vm-02, …"),
        (UiLang::Zh, Msg::HintBulkVms) => Some("vm-01, vm-02, …"),
        (UiLang::En, Msg::BtnBulkStart) => Some("Bulk start…"),
        (UiLang::Zh, Msg::BtnBulkStart) => Some("批量启动…"),
        (UiLang::En, Msg::BtnBulkStop) => Some("Bulk stop…"),
        (UiLang::Zh, Msg::BtnBulkStop) => Some("批量停止…"),
        _ => None,
    }
}

fn danger_strings_confirm_windows(lang: UiLang, msg: Msg) -> Option<&'static str> {
    match (lang, msg) {
        (UiLang::En, Msg::WinConfirmStopTitle) => Some("Confirm bulk stop"),
        (UiLang::Zh, Msg::WinConfirmStopTitle) => Some("确认批量停止"),
        (UiLang::En, Msg::WinConfirmStopBody) => {
            Some("StopVmGroup will run for the VM names above.")
        }
        (UiLang::Zh, Msg::WinConfirmStopBody) => Some("将对上述虚拟机名执行 StopVmGroup。"),
        (UiLang::En, Msg::BtnConfirmStop) => Some("Confirm stop"),
        (UiLang::Zh, Msg::BtnConfirmStop) => Some("确认停止"),
        (UiLang::En, Msg::WinConfirmStartTitle) => Some("Confirm bulk start"),
        (UiLang::Zh, Msg::WinConfirmStartTitle) => Some("确认批量启动"),
        (UiLang::En, Msg::WinConfirmStartBody) => {
            Some("StartVmGroup will run for the VM names above.")
        }
        (UiLang::Zh, Msg::WinConfirmStartBody) => Some("将对上述虚拟机名执行 StartVmGroup。"),
        (UiLang::En, Msg::BtnConfirmStart) => Some("Confirm start"),
        (UiLang::Zh, Msg::BtnConfirmStart) => Some("确认启动"),
        _ => None,
    }
}

fn danger_strings(lang: UiLang, msg: Msg) -> Option<&'static str> {
    danger_strings_card_and_actions(lang, msg).or_else(|| danger_strings_confirm_windows(lang, msg))
}
