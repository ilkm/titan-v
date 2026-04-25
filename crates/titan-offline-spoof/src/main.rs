//! CLI for golden → per-VM stamp → Sysprep hooks (Layer B). Heavy VHDX/hive work stays feature-gated.

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "titan-offline-spoof",
    version,
    about = "Offline mother-disk identity tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print build profile (default vs `offline-hive` feature).
    Status,
    /// Validate paths and emit a JSON plan for a golden VHDX (no mount when feature off).
    GoldenPrepare {
        #[arg(long)]
        vhdx: std::path::PathBuf,
        #[arg(long, default_value = "-")]
        out_json: std::path::PathBuf,
    },
    /// Record instance seed metadata for a differencing child (placeholder without hive edits).
    Stamp {
        #[arg(long)]
        vhdx: std::path::PathBuf,
        #[arg(long)]
        seed_hex: String,
    },
}

fn run_status() {
    println!("{}", titan_offline_spoof::offline_spoof_status());
}

fn run_golden_prepare(
    vhdx: std::path::PathBuf,
    out_json: std::path::PathBuf,
) -> anyhow::Result<()> {
    let plan = serde_json::json!({
        "schema": "titan.offline_spoof.golden_prepare.v1",
        "vhdx": vhdx.display().to_string(),
        "offline_hive_feature": cfg!(feature = "offline-hive"),
        "note": "Mount VHDX → offline SYSTEM/SOFTWARE keys → unmount; implement under offline-hive + admin approval.",
    });
    let text = serde_json::to_string_pretty(&plan).context("serialize plan")?;
    if out_json.as_os_str() == "-" {
        println!("{text}");
    } else {
        std::fs::write(&out_json, &text)
            .with_context(|| format!("write {}", out_json.display()))?;
    }
    Ok(())
}

fn run_stamp(vhdx: std::path::PathBuf, seed_hex: String) -> anyhow::Result<()> {
    let rec = serde_json::json!({
        "schema": "titan.offline_spoof.stamp.v1",
        "vhdx": vhdx.display().to_string(),
        "seed_hex": seed_hex.trim(),
        "status": "placeholder_no_hive_mutation",
    });
    println!("{}", serde_json::to_string_pretty(&rec)?);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Status => run_status(),
        Commands::GoldenPrepare { vhdx, out_json } => run_golden_prepare(vhdx, out_json)?,
        Commands::Stamp { vhdx, seed_hex } => run_stamp(vhdx, seed_hex)?,
    }
    Ok(())
}
