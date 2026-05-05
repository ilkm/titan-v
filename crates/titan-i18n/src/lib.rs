//! UI strings (English / Chinese) shared by **Titan Center** and **Titan Host**.
//!
//! Extend [`UiLang`] and the `strings_part*` tables for more locales or product-specific keys.

mod msg;
mod strings_host;
mod strings_part1;
mod strings_part2;
mod strings_part3;

pub use msg::Msg;
pub use titan_common::UiLang;

#[must_use]
pub fn t(lang: UiLang, msg: Msg) -> &'static str {
    strings_part1::translate(lang, msg)
        .or_else(|| strings_part2::translate(lang, msg))
        .or_else(|| strings_part3::translate(lang, msg))
        .or_else(|| strings_host::translate(lang, msg))
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
pub fn hp_env_listen_applied(lang: UiLang) -> String {
    match lang {
        UiLang::En => "Applied environment variable TITAN_HOST_LISTEN.".into(),
        UiLang::Zh => "已应用环境变量 TITAN_HOST_LISTEN".into(),
    }
}

#[must_use]
pub fn hp_control_listening(lang: UiLang, listen: &str) -> String {
    match lang {
        UiLang::En => format!("Control plane listening on {listen}"),
        UiLang::Zh => format!("控制面已监听 {listen}"),
    }
}

#[must_use]
pub fn hp_quic_pairing_remaining(lang: UiLang, secs: u64) -> String {
    match lang {
        UiLang::En => format!("Pairing window open: {secs}s left to auto-trust new centers."),
        UiLang::Zh => format!("配对窗口开启中：剩余 {secs} 秒，自动信任新中控。"),
    }
}
