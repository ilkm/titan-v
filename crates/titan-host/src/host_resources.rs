//! Blocking host CPU / memory / network counters for [`ControlRequest::HostResourceSnapshot`].

use std::sync::Mutex;
use std::time::Instant;

use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use titan_common::HostResourceStats;

static NET_PREV: Mutex<Option<(Instant, u64, u64)>> = Mutex::new(None);
static DISK_PREV: Mutex<Option<(Instant, u64, u64)>> = Mutex::new(None);

fn net_rates_bps() -> (u64, u64) {
    let mut nets = Networks::new_with_refreshed_list();
    nets.refresh(false);
    let mut rx = 0u64;
    let mut tx = 0u64;
    for (_, d) in nets.iter() {
        rx = rx.saturating_add(d.total_received());
        tx = tx.saturating_add(d.total_transmitted());
    }
    let now = Instant::now();
    let mut guard = NET_PREV.lock().unwrap_or_else(|e| e.into_inner());
    let out = match *guard {
        None => (0u64, 0u64),
        Some((t0, r0, t0_tx)) => {
            let dt = now.saturating_duration_since(t0).as_secs_f64().max(0.05);
            let drx = rx.saturating_sub(r0);
            let dtx = tx.saturating_sub(t0_tx);
            ((drx as f64 / dt) as u64, (dtx as f64 / dt) as u64)
        }
    };
    *guard = Some((now, rx, tx));
    out
}

fn disk_io_bps() -> (u64, u64) {
    let disks = Disks::new_with_refreshed_list();
    let mut read_sum = 0u64;
    let mut write_sum = 0u64;
    for disk in disks.list() {
        let u = disk.usage();
        read_sum = read_sum.saturating_add(u.total_read_bytes);
        write_sum = write_sum.saturating_add(u.total_written_bytes);
    }
    let now = Instant::now();
    let mut guard = DISK_PREV.lock().unwrap_or_else(|e| e.into_inner());
    let out = match *guard {
        None => (0u64, 0u64),
        Some((t0, r0, w0)) => {
            let dt = now.saturating_duration_since(t0).as_secs_f64().max(0.05);
            let dr = read_sum.saturating_sub(r0);
            let dw = write_sum.saturating_sub(w0);
            ((dr as f64 / dt) as u64, (dw as f64 / dt) as u64)
        }
    };
    *guard = Some((now, read_sum, write_sum));
    out
}

/// Samples the local OS (blocking; call from `spawn_blocking`).
#[must_use]
pub fn collect_blocking() -> HostResourceStats {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    let cpu = sys.global_cpu_usage().clamp(0.0, 100.0);
    let cpu_permille = (cpu * 10.0).round() as u32;

    sys.refresh_memory();
    let mem_used_bytes = sys.used_memory();
    let mem_total_bytes = sys.total_memory().max(1);

    let (net_down_bps, net_up_bps) = net_rates_bps();
    let (disk_read_bps, disk_write_bps) = disk_io_bps();

    HostResourceStats {
        cpu_permille,
        mem_used_bytes,
        mem_total_bytes,
        net_down_bps,
        net_up_bps,
        disk_read_bps,
        disk_write_bps,
    }
}
