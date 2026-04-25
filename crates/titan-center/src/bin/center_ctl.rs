//! Push host UI JSON from the center SQLite store to a single host (control-plane TCP).

use anyhow::Context;
use clap::{Parser, Subcommand};
use titan_center::app::device_store;
use titan_center::app::net_client::exchange_one;
use titan_common::ControlRequest;

#[derive(Parser)]
#[command(name = "titan-center-ctl", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the resolved `devices.sqlite` path (same file as the GUI).
    DbPath,
    /// Read `host_managed_config` for `device_id` and send `ApplyHostUiPersistJson` to that host's `addr`.
    PushHostConfig { device_id: String },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::DbPath => println!("{}", device_store::registration_db_path().display()),
        Cmd::PushHostConfig { device_id } => {
            let db = device_store::registration_db_path();
            let json = device_store::load_host_managed_config(&db, &device_id)
                .context("sqlite read")?
                .with_context(|| format!("no host_managed_config row for device_id={device_id}"))?;
            let addr = device_store::addr_for_device_id(&db, &device_id)
                .context("sqlite addr")?
                .with_context(|| format!("no registered_devices row for device_id={device_id}"))?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let res = rt.block_on(exchange_one(
                &addr,
                &ControlRequest::ApplyHostUiPersistJson { json },
            ))?;
            println!("{res:?}");
        }
    }
    Ok(())
}
