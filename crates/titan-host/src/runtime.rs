//! Per-VM Lua execution (Phase 2) with a **bounded queue** fed by `LoadScriptVm`.

use std::sync::mpsc as std_mpsc;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use titan_scripts::ScriptEngine;
use tokio::sync::{mpsc, Mutex};

/// One script execution job from the control plane.
#[derive(Debug)]
pub struct ScriptJob {
    pub vm_name: String,
    pub source: String,
}

/// Bounded queue depth for `LoadScriptVm` (backpressure).
pub const SCRIPT_QUEUE_CAPACITY: usize = 256;

/// Executes `source` in an isolated engine (VM name reserved for future per-VM sandbox tables).
pub fn exec_vm_chunk(vm_name: &str, source: &str) -> Result<(), String> {
    let _ = vm_name;
    let eng = ScriptEngine::new().map_err(|e| format!("lua init: {e}"))?;
    eng.exec_chunk(source).map_err(|e| format!("lua exec: {e}"))
}

/// Runs [`exec_vm_chunk`] on a joinable std thread with a hard wall-clock limit (Lua keeps running on timeout).
pub fn exec_vm_chunk_with_timeout(
    vm_name: &str,
    source: &str,
    budget: Duration,
) -> Result<(), String> {
    let (tx, rx) = std_mpsc::channel::<Result<(), String>>();
    let vm = vm_name.to_string();
    let src = source.to_string();
    let handle = std::thread::spawn(move || {
        let r = exec_vm_chunk(&vm, &src);
        let _ = tx.send(r);
    });
    match rx.recv_timeout(budget) {
        Ok(r) => {
            let _ = handle.join();
            r
        }
        Err(std_mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "script exceeded {}s wall time (Lua thread may still be running)",
            budget.as_secs()
        )),
        Err(std_mpsc::RecvTimeoutError::Disconnected) => {
            let _ = handle.join();
            Err("script thread disconnected".into())
        }
    }
}

/// Budget for a single `exec_chunk` when wrapped in `tokio::time::timeout`.
pub const DEFAULT_SCRIPT_EXEC_TIMEOUT: Duration = Duration::from_secs(8);

/// Consumes script jobs: **per-VM serial** (one in-flight script per `vm_name`) + wall-clock timeout.
pub async fn script_worker(
    mut rx: mpsc::Receiver<ScriptJob>,
    vm_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
) {
    while let Some(job) = rx.recv().await {
        let vm = job.vm_name.clone();
        let lock = vm_locks
            .entry(vm.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let _guard = lock.lock().await;
        let source = job.source;
        let vm_for_log = vm.clone();
        let res = tokio::task::spawn_blocking(move || {
            exec_vm_chunk_with_timeout(&vm, &source, DEFAULT_SCRIPT_EXEC_TIMEOUT)
        })
        .await;
        match res {
            Ok(Ok(())) => tracing::info!(vm = %vm_for_log, "script job ok"),
            Ok(Err(e)) => tracing::warn!(vm = %vm_for_log, error = %e, "script job failed"),
            Err(e) => tracing::warn!(vm = %vm_for_log, error = %e, "script join failed"),
        }
    }
}

/// Low-frequency tick for future multi-VM scheduling hooks.
pub fn spawn_coordinator_ticks() {
    tokio::spawn(async {
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
            tracing::trace!("host coordinator tick");
        }
    });
}

#[cfg(test)]
mod lua_backpressure_contract {
    #[test]
    fn script_queue_holds_need_md_scale_window() {
        assert!(
            std::cmp::Ordering::Less != super::SCRIPT_QUEUE_CAPACITY.cmp(&40),
            "Lua script queue should admit at least 40 pending jobs for scaled orchestration (see need.md)"
        );
    }
}
