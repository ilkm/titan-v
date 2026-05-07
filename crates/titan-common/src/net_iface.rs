use std::collections::HashSet;
use std::net::Ipv4Addr;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "windows")]
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanIpv4Row {
    pub ip: Ipv4Addr,
    pub iface: String,
}

pub fn list_physical_lan_ipv4_rows() -> Vec<LanIpv4Row> {
    #[cfg(target_os = "windows")]
    if let Some(rows) = windows_cached_rows() {
        return rows;
    }
    let rows = collect_physical_lan_ipv4_rows();
    #[cfg(target_os = "windows")]
    windows_store_rows(&rows);
    rows
}

fn collect_physical_lan_ipv4_rows() -> Vec<LanIpv4Row> {
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
        if iface_matches_physical_policy(&iface.name, allowlist.as_ref()) {
            rows.push(LanIpv4Row {
                ip: v4.ip,
                iface: iface.name,
            });
        }
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
    let mut names = HashSet::new();
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return None;
    };
    for iface in ifaces {
        let name = iface.name;
        if is_macos_hardware_iface_name(&name) {
            let _ = names.insert(name);
        }
    }
    (!names.is_empty()).then_some(names)
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
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn physical_iface_allowlist() -> Option<HashSet<String>> {
    None
}

#[cfg(target_os = "macos")]
fn is_macos_hardware_iface_name(name: &str) -> bool {
    let mut parts = name.splitn(2, |c: char| c.is_ascii_digit());
    let prefix = parts.next().unwrap_or_default();
    let suffix = &name[prefix.len()..];
    prefix == "en" && !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
}

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct WindowsLanCache {
    at: Instant,
    rows: Vec<LanIpv4Row>,
}

#[cfg(target_os = "windows")]
fn windows_lan_cache() -> &'static Mutex<Option<WindowsLanCache>> {
    static CACHE: OnceLock<Mutex<Option<WindowsLanCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "windows")]
fn windows_cached_rows() -> Option<Vec<LanIpv4Row>> {
    const CACHE_TTL_MS: u128 = 3000;
    let guard = windows_lan_cache().lock().ok()?;
    let cache = guard.as_ref()?;
    (cache.at.elapsed().as_millis() <= CACHE_TTL_MS).then(|| cache.rows.clone())
}

#[cfg(target_os = "windows")]
fn windows_store_rows(rows: &[LanIpv4Row]) {
    if let Ok(mut guard) = windows_lan_cache().lock() {
        *guard = Some(WindowsLanCache {
            at: Instant::now(),
            rows: rows.to_vec(),
        });
    }
}

#[cfg(target_os = "linux")]
fn is_linux_physical_iface(path: &std::path::Path) -> bool {
    let Ok(link) = std::fs::read_link(path) else {
        return false;
    };
    !link.to_string_lossy().contains("/virtual/")
}
