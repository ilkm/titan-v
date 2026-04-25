use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::Duration;

use titan_common::VmProvisionPlan;
use tokio::sync::watch;

use crate::batch::run_provision_plans;
use crate::config::expand_vm_plans;
use crate::host_font;
use crate::serve::{run_serve, AgentBindingsSpec, HostAnnounceConfig};

use super::model::{HostApp, HostUiPersist, ServeRun, PERSIST_KEY};

fn serve_thread_main(
    listen: SocketAddr,
    spec: AgentBindingsSpec,
    announce: HostAnnounceConfig,
    shutdown_rx: watch::Receiver<bool>,
    persist_apply_tx: Option<std::sync::mpsc::Sender<crate::ui_persist::HostUiPersist>>,
) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("tokio runtime: {e}");
            return;
        }
    };
    if let Err(e) = rt.block_on(run_serve(
        listen,
        spec,
        announce,
        shutdown_rx,
        persist_apply_tx,
    )) {
        tracing::warn!(error = %e, "serve thread ended with error");
    } else {
        tracing::info!("serve thread ended");
    }
}

impl HostApp {
    /// `initial_tray`: build with [`titan_tray::build_host_tray_icon`] in the `eframe::run_native` closure
    /// **before** constructing the app (matches tray-icon's egui example; avoids macOS first-frame ordering issues).
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_tray: Option<titan_tray::TrayIcon>,
    ) -> Self {
        host_font::install_cjk_fonts(&cc.egui_ctx);

        let json_opt = cc.storage.and_then(|s| s.get_string(PERSIST_KEY));
        let mut persist: HostUiPersist = json_opt
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let mut env_listen_hint = None;
        if let Ok(s) = std::env::var("TITAN_HOST_LISTEN") {
            if s.parse::<SocketAddr>().is_ok() {
                persist.listen = s;
                env_listen_hint = Some("已应用环境变量 TITAN_HOST_LISTEN".into());
            }
        }

        let (persist_apply_tx, persist_apply_rx) = std::sync::mpsc::channel();
        Self {
            ctx: cc.egui_ctx.clone(),
            really_quitting: false,
            hidden_to_tray: false,
            _tray: initial_tray,
            serve_run: None,
            persist_apply_tx: Some(persist_apply_tx),
            persist_apply_rx,
            persist,
            active_tab: 0,
            status_line: String::new(),
            provision_log: Vec::new(),
            provision_rx: None,
            env_listen_hint,
            binding_vm: String::new(),
            binding_addr: String::new(),
            initial_serve_attempted: false,
            boot_window_focus_once: false,
        }
    }

    pub(crate) fn start_serve(&mut self) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }

        let listen = match self.persist.parse_listen() {
            Ok(a) => a,
            Err(e) => {
                self.status_line = e;
                return;
            }
        };

        let spec = match self.persist.bindings_spec() {
            Ok(s) => s,
            Err(e) => {
                self.status_line = e;
                return;
            }
        };

        let announce = self.persist.to_announce();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let persist_tx = self.persist_apply_tx.clone();
        let join = std::thread::Builder::new()
            .name("titan-host-serve".into())
            .spawn(move || serve_thread_main(listen, spec, announce, shutdown_rx, persist_tx))
            .expect("spawn serve thread");

        self.serve_run = Some(ServeRun { shutdown_tx, join });
        self.status_line = format!("控制面已监听 {}", self.persist.listen);
    }

    pub(crate) fn drain_provision_log(&mut self) {
        let Some(rx) = self.provision_rx.as_ref() else {
            return;
        };
        while let Ok(line) = rx.try_recv() {
            self.provision_log.push(line);
            if self.provision_log.len() > 400 {
                self.provision_log
                    .drain(0..self.provision_log.len().saturating_sub(300));
            }
        }
    }

    pub(crate) fn run_provision_clicked(&mut self, dry_run: bool) {
        let Some(plans) = expanded_batch_plans_or_status(&self.persist, &mut self.status_line)
        else {
            return;
        };

        let timeout = Duration::from_secs(self.persist.batch_timeout_secs.max(1));
        let fail_fast = self.persist.batch_fail_fast;
        let (tx, rx) = mpsc::channel();
        self.provision_rx = Some(rx);
        self.provision_log.clear();
        let phase = if dry_run {
            "预检 (dry-run)"
        } else {
            "创建"
        };
        let _ = tx.send(format!("开始{phase} — 共 {} 台", plans.len()));

        std::thread::Builder::new()
            .name("titan-host-provision".into())
            .spawn(move || {
                provision_plans_thread(tx, plans, timeout, fail_fast, dry_run);
            })
            .expect("spawn provision");

        self.ctx.request_repaint();
    }
}

fn expanded_batch_plans_or_status(
    persist: &HostUiPersist,
    status: &mut String,
) -> Option<Vec<VmProvisionPlan>> {
    let plans = match expand_vm_plans(&persist.batch_vm, &persist.batch_vm_group) {
        Ok(p) => p,
        Err(e) => {
            *status = e.to_string();
            return None;
        }
    };
    if plans.is_empty() {
        *status = "没有可创建的虚拟机：请添加「显式 VM」或「VM 组」".into();
        return None;
    }
    Some(plans)
}

fn provision_plans_thread(
    tx: mpsc::Sender<String>,
    plans: Vec<VmProvisionPlan>,
    timeout: Duration,
    fail_fast: bool,
    dry_run: bool,
) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(format!("tokio runtime: {e}"));
            return;
        }
    };
    let res = rt.block_on(run_provision_plans(plans, timeout, fail_fast, dry_run));
    let msg = match res {
        Ok(()) => "批量任务结束".into(),
        Err(e) => format!("批量失败: {e}"),
    };
    let _ = tx.send(msg);
}
