use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanIpv4Row {
    pub ip: Ipv4Addr,
    pub iface: String,
}

pub fn list_physical_lan_ipv4_rows() -> Vec<LanIpv4Row> {
    let mut rows = Vec::new();
    let allowlist = physical_iface_allowlist();
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return rows;
    };
    for iface in ifaces {
        if iface.is_loopback() {
            continue;
        }
        let if_addrs::IfAddr::V4(v4) = iface.addr else {
            continue;
        };
        if !is_usable_lan_ipv4(v4.ip) {
            continue;
        }
        if !iface_matches_physical_policy(&iface.name, allowlist.as_ref()) {
            continue;
        }
        rows.push(LanIpv4Row {
            ip: v4.ip,
            iface: iface.name,
        });
    }
    rows.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.iface.cmp(&b.iface)));
    rows.dedup_by(|a, b| a.ip == b.ip && a.iface == b.iface);
    rows
}

fn is_usable_lan_ipv4(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified() && !ip.is_loopback() && !ip.is_link_local() && !ip.is_multicast()
}

fn iface_matches_physical_policy(name: &str, allowlist: Option<&HashSet<String>>) -> bool {
    if let Some(list) = allowlist {
        return list.contains(name);
    }
    !is_virtual_iface_name(name)
}

fn is_virtual_iface_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    virtual_iface_needles()
        .iter()
        .any(|needle| n.contains(needle))
}

fn virtual_iface_needles() -> [&'static str; 21] {
    [
        "virtual",
        "vmware",
        "vbox",
        "hyper-v",
        "hyperv",
        "vethernet",
        "docker",
        "wsl",
        "npcap",
        "loopback",
        "tunnel",
        "bridge",
        "br-",
        "tap",
        "tun",
        "utun",
        "tailscale",
        "zerotier",
        "wireguard",
        "hamachi",
        "vpn",
    ]
}

#[cfg(target_os = "macos")]
fn physical_iface_allowlist() -> Option<HashSet<String>> {
    parse_device_names(run_stdout("networksetup", &["-listallhardwareports"])?)
}

#[cfg(target_os = "linux")]
fn physical_iface_allowlist() -> Option<HashSet<String>> {
    let mut names = HashSet::new();
    let base = std::path::Path::new("/sys/class/net");
    let entries = std::fs::read_dir(base).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_linux_physical_iface(&entry.path()) {
            let _ = names.insert(name);
        }
    }
    (!names.is_empty()).then_some(names)
}

#[cfg(target_os = "windows")]
fn physical_iface_allowlist() -> Option<HashSet<String>> {
    parse_lines(run_stdout(
        "powershell.exe",
        &[
            "-NoProfile",
            "-Command",
            "Get-NetAdapter -Physical | Select-Object -ExpandProperty Name",
        ],
    )?)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn physical_iface_allowlist() -> Option<HashSet<String>> {
    None
}

fn run_stdout(bin: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(bin).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(target_os = "macos")]
fn parse_device_names(text: String) -> Option<HashSet<String>> {
    let mut names = HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("Device: ") {
            let _ = names.insert(name.trim().to_string());
        }
    }
    (!names.is_empty()).then_some(names)
}

#[cfg(target_os = "linux")]
fn is_linux_physical_iface(path: &std::path::Path) -> bool {
    let Ok(link) = std::fs::read_link(path) else {
        return false;
    };
    !link.to_string_lossy().contains("/virtual/")
}

#[cfg(target_os = "windows")]
fn parse_lines(text: String) -> Option<HashSet<String>> {
    let mut names = HashSet::new();
    for line in text.lines() {
        let name = line.trim();
        if name.is_empty() || name.eq_ignore_ascii_case("Name") {
            continue;
        }
        let _ = names.insert(name.to_string());
    }
    (!names.is_empty()).then_some(names)
}
