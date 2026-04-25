//! UI strings (English / Chinese). Extend `UiLang` + string tables for more locales.

mod msg;
mod strings_part1;
mod strings_part2;
mod strings_part3;
mod strings_part4;

pub use msg::Msg;

use serde::{Deserialize, Serialize};

/// Display language for the center UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLang {
    #[default]
    En,
    Zh,
}

#[must_use]
pub fn t(lang: UiLang, msg: Msg) -> &'static str {
    strings_part1::translate(lang, msg)
        .or_else(|| strings_part2::translate(lang, msg))
        .or_else(|| strings_part3::translate(lang, msg))
        .or_else(|| strings_part4::translate(lang, msg))
        .unwrap_or("(i18n)")
}

#[must_use]
pub fn host_running_windows_line(lang: UiLang, n: u32) -> String {
    match lang {
        UiLang::En => format!("{n} running windows"),
        UiLang::Zh => format!("{n} 个运行窗口"),
    }
}

#[must_use]
pub fn log_host_responded(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Host responded (Hello or Ping).".into(),
        UiLang::Zh => "宿主已响应（Hello 或 Ping）。".into(),
    }
}

#[must_use]
pub fn log_list_vms(lang: UiLang, n: usize) -> String {
    match lang {
        UiLang::En => format!("ListVms: {n} VM(s)."),
        UiLang::Zh => format!("ListVms：{n} 台虚拟机。"),
    }
}

#[must_use]
pub fn log_stop_vm_group(lang: UiLang, ok: u32, failures: usize) -> String {
    match lang {
        UiLang::En => format!("StopVmGroup: ok={ok}, failures={failures}"),
        UiLang::Zh => format!("StopVmGroup：成功 {ok}，失败 {failures}"),
    }
}

#[must_use]
pub fn log_start_vm_group(lang: UiLang, ok: u32, failures: usize) -> String {
    match lang {
        UiLang::En => format!("StartVmGroup: ok={ok}, failures={failures}"),
        UiLang::Zh => format!("StartVmGroup：成功 {ok}，失败 {failures}"),
    }
}

#[must_use]
pub fn log_spoof_apply(lang: UiLang, dry_run: bool, steps: &str, notes: &str) -> String {
    match lang {
        UiLang::En => format!("ApplySpoofProfile dry_run={dry_run}: {steps} | {notes}"),
        UiLang::Zh => format!("ApplySpoofProfile dry_run={dry_run}：{steps} | {notes}"),
    }
}

#[must_use]
pub fn log_lan_host_announced(lang: UiLang, label: &str, addr: &str) -> String {
    match lang {
        UiLang::En => format!("LAN: host {label} announced at {addr} (device list updated)."),
        UiLang::Zh => format!("局域网：主机 {label} 在 {addr} 宣告（设备列表已更新）。"),
    }
}

#[must_use]
pub fn log_request_failed(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Request failed.".into(),
        UiLang::Zh => "请求失败。".into(),
    }
}

#[must_use]
pub fn log_spoof_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Spoof apply cancelled.".into(),
        UiLang::Zh => "已取消应用伪装。".into(),
    }
}

#[must_use]
pub fn log_spoof_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "ApplySpoofProfile dispatched.".into(),
        UiLang::Zh => "已发送 ApplySpoofProfile。".into(),
    }
}

#[must_use]
pub fn log_bulk_stop_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Bulk stop cancelled.".into(),
        UiLang::Zh => "已取消批量停止。".into(),
    }
}

#[must_use]
pub fn log_bulk_start_cancelled(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Bulk start cancelled.".into(),
        UiLang::Zh => "已取消批量启动。".into(),
    }
}

#[must_use]
pub fn log_no_vm_names(lang: UiLang) -> String {
    match lang {
        UiLang::En => "No VM names parsed (comma-separated).".into(),
        UiLang::Zh => "未解析到虚拟机名（请用逗号分隔）。".into(),
    }
}

#[must_use]
pub fn log_stop_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "StopVmGroup dispatched.".into(),
        UiLang::Zh => "已发送 StopVmGroup。".into(),
    }
}

#[must_use]
pub fn log_start_dispatched(lang: UiLang) -> String {
    match lang {
        UiLang::En => "StartVmGroup dispatched.".into(),
        UiLang::Zh => "已发送 StartVmGroup。".into(),
    }
}

#[must_use]
pub fn fmt_slot_grid_header(lang: UiLang, rows: usize, host: &str) -> String {
    match lang {
        UiLang::En => format!("{rows} rows · selected host: {host}"),
        UiLang::Zh => format!("{rows} 行 · 当前主机：{host}"),
    }
}

#[must_use]
pub fn fmt_slot_line_empty(lang: UiLang, host: &str, i: usize) -> String {
    let empty = t(lang, Msg::SlotEmpty);
    format!("{host} · {i:04} — {empty}")
}

#[must_use]
pub fn fmt_slot_line_vm(
    _lang: UiLang,
    host: &str,
    i: usize,
    vm_name: &str,
    state_dbg: &str,
) -> String {
    format!("{host} · {i:04} → {vm_name} · {state_dbg}")
}
