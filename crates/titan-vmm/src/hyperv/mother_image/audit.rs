//! Audit JSONL and dry-run step markers.

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use titan_common::{Error, Result, VmSpoofProfile};

pub(super) fn append_audit_line(path: &Path, record: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(Error::Io)?;
    let line = serde_json::to_string(record).map_err(|e| Error::HyperVRejected {
        message: format!("audit json: {e}"),
    })?;
    writeln!(f, "{line}").map_err(Error::Io)?;
    Ok(())
}

pub(super) fn run_or_record(
    vm: &str,
    step: &str,
    script: &str,
    dry_run: bool,
    audit: Option<&Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    let rec = serde_json::json!({
        "vm": vm,
        "step": step,
        "dry_run": dry_run,
    });
    if let Some(p) = audit {
        let _ = append_audit_line(p, &rec);
    }
    if dry_run {
        tracing::info!(%vm, %step, "spoof dry-run (skipped execution)");
        steps_out.push(format!("{step}(dry-run)"));
        return Ok(());
    }
    super::ps::run_ps_job(vm, script)?;
    steps_out.push(format!("{step}(applied)"));
    Ok(())
}

/// Append audit JSONL + step marker without running PowerShell (caller runs custom `Command` if needed).
pub(super) fn audit_only(
    vm: &str,
    step: &str,
    dry_run: bool,
    audit: Option<&Path>,
    steps_out: &mut Vec<String>,
) -> Result<()> {
    let rec = serde_json::json!({
        "vm": vm,
        "step": step,
        "dry_run": dry_run,
    });
    if let Some(p) = audit {
        append_audit_line(p, &rec)?;
    }
    if dry_run {
        tracing::info!(%vm, %step, "spoof dry-run (skipped execution)");
        steps_out.push(format!("{step}(dry-run)"));
    } else {
        steps_out.push(format!("{step}(applied)"));
    }
    Ok(())
}

pub(super) fn audit_path_from(profile: &VmSpoofProfile) -> Option<PathBuf> {
    profile
        .audit_log_path
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}
