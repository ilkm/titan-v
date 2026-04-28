use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::Duration;

use titan_common::{UiLang, VmProvisionPlan};
use tokio::sync::watch;

use crate::batch::run_provision_plans;
use crate::config::expand_vm_plans;
use crate::host_font;
use crate::serve::{run_serve, AgentBindingsSpec, HostAnnounceConfig};

use super::model::{HostApp, HostUiPersist, ServeRun, PERSIST_KEY};
use super::theme::apply_host_chrome_theme;

fn host_try_build_serve_runtime() -> Option<tokio::runtime::Runtime> {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => Some(r),
        Err(e) => {
            tracing::error!("tokio runtime: {e}");
            None
        }
    }
}

fn serve_thread_main(
    listen: SocketAddr,
    spec: AgentBindingsSpec,
    announce: HostAnnounceConfig,
    shutdown_rx: watch::Receiver<bool>,
    persist_apply_tx: Option<std::sync::mpsc::Sender<crate::ui_persist::HostUiPersist>>,
    lang_apply_tx: Option<std::sync::mpsc::Sender<UiLang>>,
) {
    let Some(rt) = host_try_build_serve_runtime() else {
        return;
    };
    let res = rt.block_on(run_serve(
        listen,
        spec,
        announce,
        shutdown_rx,
        persist_apply_tx,
        lang_apply_tx,
    ));
    if let Err(e) = res {
        tracing::warn!(error = %e, "serve thread ended with error");
    } else {
        tracing::info!("serve thread ended");
    }
}

impl HostApp {
    /// `initial_tray`: build with [`titan_tray::build_host_tray_icon`] and the persisted [`UiLang`](titan_common::UiLang) in the `eframe::run_native` closure
    /// **before** constructing the app (matches tray-icon's egui example; avoids macOS first-frame ordering issues).
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_tray: Option<titan_tray::TrayIcon>,
    ) -> Self {
        host_font::install_cjk_fonts(&cc.egui_ctx);
        apply_host_chrome_theme(&cc.egui_ctx);

        let json_opt = cc.storage.and_then(|s| s.get_string(PERSIST_KEY));
        let mut persist: HostUiPersist = json_opt
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let mut env_listen_hint = None;
        if let Ok(s) = std::env::var("TITAN_HOST_LISTEN") {
            if s.parse::<SocketAddr>().is_ok() {
                persist.listen = s;
                env_listen_hint = Some(crate::titan_i18n::hp_env_listen_applied(persist.ui_lang));
            }
        }

        let (persist_apply_tx, persist_apply_rx) = std::sync::mpsc::channel();
        let (lang_apply_tx, lang_apply_rx) = std::sync::mpsc::channel();
        Self {
            ctx: cc.egui_ctx.clone(),
            really_quitting: false,
            hidden_to_tray: false,
            _tray: initial_tray,
            tray_glyph_lang: persist.ui_lang,
            serve_run: None,
            persist_apply_tx: Some(persist_apply_tx),
            persist_apply_rx,
            lang_apply_tx: Some(lang_apply_tx),
            lang_apply_rx,
            persist,
            active_tab: 0,
            status_line: String::new(),
            provision_log: Vec::new(),
            provision_rx: None,
            env_listen_hint,
            initial_serve_attempted: false,
            boot_window_focus_once: false,
            settings_open: false,
            settings_lang_btn_rect: None,
        }
    }

    fn start_serve_resolve(
        &mut self,
    ) -> Option<(SocketAddr, AgentBindingsSpec, HostAnnounceConfig)> {
        let listen = match self.persist.parse_listen() {
            Ok(a) => a,
            Err(e) => {
                self.status_line = e;
                return None;
            }
        };
        let spec = match HostUiPersist::build_agent_bindings_spec() {
            Ok(s) => s,
            Err(e) => {
                self.status_line = e;
                return None;
            }
        };
        Some((listen, spec, self.persist.to_announce()))
    }

    pub(crate) fn start_serve(&mut self) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }
        let Some((listen, spec, announce)) = self.start_serve_resolve() else {
            return;
        };
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let persist_tx = self.persist_apply_tx.clone();
        let lang_tx = self.lang_apply_tx.clone();
        let join = std::thread::Builder::new()
            .name("titan-host-serve".into())
            .spawn(move || {
                serve_thread_main(listen, spec, announce, shutdown_rx, persist_tx, lang_tx)
            })
            .expect("spawn serve thread");
        self.serve_run = Some(ServeRun { shutdown_tx, join });
        self.status_line =
            crate::titan_i18n::hp_control_listening(self.persist.ui_lang, &self.persist.listen);
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
        let ui_lang = self.persist.ui_lang;
        let (tx, rx) = mpsc::channel();
        self.provision_rx = Some(rx);
        self.provision_log.clear();
        let banner = crate::titan_i18n::hp_provision_start_banner(
            self.persist.ui_lang,
            dry_run,
            plans.len(),
        );
        let _ = tx.send(banner);

        std::thread::Builder::new()
            .name("titan-host-provision".into())
            .spawn(move || {
                provision_plans_thread(tx, plans, timeout, fail_fast, dry_run, ui_lang);
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
        *status = crate::titan_i18n::hp_batch_no_plans(persist.ui_lang);
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
    ui_lang: UiLang,
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
        Ok(()) => crate::titan_i18n::hp_provision_done_ok(ui_lang),
        Err(e) => crate::titan_i18n::hp_provision_done_err(ui_lang, &e.to_string()),
    };
    let _ = tx.send(msg);
}
