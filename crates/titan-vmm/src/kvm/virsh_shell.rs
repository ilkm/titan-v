//! Optional **libvirt shell** integration via `virsh` (Linux only). No shell interpolation — argv only.

use std::process::{Command, Stdio};

use titan_common::{state::VmPowerState, Error, Result};

/// Known trailing state tokens from `virsh list --all` (longest first for suffix match).
const VIRSH_STATE_SUFFIXES: &[(&str, VmPowerState)] = &[
    ("shut off", VmPowerState::Off),
    ("shutting down", VmPowerState::Off),
    ("in shutdown", VmPowerState::Off),
    ("pmsuspended", VmPowerState::Paused),
    ("running", VmPowerState::Running),
    ("paused", VmPowerState::Paused),
    ("blocked", VmPowerState::Unknown),
    ("crashed", VmPowerState::Off),
    ("closed", VmPowerState::Off),
    ("shutdown", VmPowerState::Off),
    ("idle", VmPowerState::Unknown),
];

/// Returns true when `virsh --version` exits zero (libvirt client tools on PATH).
#[must_use]
pub fn virsh_version_available_blocking() -> bool {
    #[cfg(target_os = "linux")]
    {
        Command::new("virsh")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

fn run_virsh(args: &[&str]) -> Result<std::process::Output> {
    Command::new("virsh")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Error::VmmRejected {
            message: format!("failed to spawn virsh: {e}"),
        })
}

fn validate_domain_token(name: &str) -> Result<()> {
    let t = name.trim();
    if t.is_empty() {
        return Err(Error::VmmRejected {
            message: "domain name must not be empty".into(),
        });
    }
    let ok = t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if !ok {
        return Err(Error::VmmRejected {
            message: "domain name contains disallowed characters".into(),
        });
    }
    Ok(())
}

fn format_virsh_fail(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let mut s = format!("virsh exit code: {:?}", code);
    if !stdout.is_empty() {
        s.push_str("\nstdout:\n");
        s.push_str(&stdout);
    }
    if !stderr.is_empty() {
        s.push_str("\nstderr:\n");
        s.push_str(&stderr);
    }
    s
}

/// Parses stdout of `virsh list --all` into `(name, state)` rows.
#[must_use]
pub fn parse_virsh_list_all(stdout: &str) -> Vec<(String, VmPowerState)> {
    let mut out = Vec::new();
    for raw in stdout.lines() {
        let line = raw.trim();
        if line.is_empty()
            || line.starts_with("Id")
            || line.chars().all(|c| c == '-' || c.is_whitespace())
        {
            continue;
        }
        let Some(parsed) = parse_one_list_line(line) else {
            continue;
        };
        out.push(parsed);
    }
    out
}

fn parse_one_list_line(line: &str) -> Option<(String, VmPowerState)> {
    let mut rest = line.trim();
    let mut state = VmPowerState::Unknown;
    for (suffix, st) in VIRSH_STATE_SUFFIXES {
        if let Some(prefix) = rest.strip_suffix(suffix) {
            let prefix = prefix.trim_end();
            if prefix.is_empty() {
                return None;
            }
            rest = prefix;
            state = *st;
            break;
        }
    }
    let mut parts = rest.split_whitespace();
    let _id = parts.next()?;
    let name = parts.collect::<Vec<_>>().join(" ");
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), state))
}

/// Lists libvirt domains when `virsh` is available; caller should probe first for empty fallback.
pub fn list_domains_blocking() -> Result<Vec<(String, VmPowerState)>> {
    let out = run_virsh(&["list", "--all"])?;
    if !out.status.success() {
        return Err(Error::VmmRejected {
            message: format_virsh_fail(out.status.code(), &out.stdout, &out.stderr),
        });
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(parse_virsh_list_all(&text))
}

/// `virsh start` or `virsh destroy` (hard off; mirrors Hyper-V `-Force` stop intent).
pub fn domain_set_power_blocking(domain: &str, start: bool) -> Result<()> {
    validate_domain_token(domain)?;
    let args: [&str; 2] = if start {
        ["start", domain.trim()]
    } else {
        ["destroy", domain.trim()]
    };
    let out = run_virsh(&args)?;
    if out.status.success() {
        return Ok(());
    }
    Err(Error::VmmRejected {
        message: format_virsh_fail(out.status.code(), &out.stdout, &out.stderr),
    })
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn parse_sample_table() {
        let sample = r" Id    Name                           State
----------------------------------------------------
 2     ubuntu2204                     running
 -     debian12                       shut off
";
        let rows = parse_virsh_list_all(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "ubuntu2204");
        assert_eq!(rows[0].1, VmPowerState::Running);
        assert_eq!(rows[1].0, "debian12");
        assert_eq!(rows[1].1, VmPowerState::Off);
    }

    #[test]
    fn reject_bad_domain_for_power() {
        let err = domain_set_power_blocking("vm;rm -rf /", true).unwrap_err();
        assert!(matches!(err, Error::VmmRejected { .. }), "{err}");
    }
}
