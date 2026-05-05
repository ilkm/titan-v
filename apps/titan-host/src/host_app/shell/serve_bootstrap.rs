use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::watch;

use crate::agent_binding_table::AgentBindingTable;
use crate::host_font;
use crate::serve::{HostAnnounceConfig, ServeSecurity, ServeUiChannels, run_serve};

use crate::host_app::model::{HostApp, HostSecurity, HostUiPersist, PERSIST_KEY, ServeRun};
use crate::host_app::ui::theme::apply_host_chrome_theme;
use crate::serve::VmWindowReloadMsg;
use std::sync::mpsc;
use titan_common::UiLang;

mod security;

use security::{build_serve_security, init_host_security};

struct HostAppChannels {
    persist_apply_tx: mpsc::Sender<HostUiPersist>,
    persist_apply_rx: mpsc::Receiver<HostUiPersist>,
    lang_apply_tx: mpsc::Sender<UiLang>,
    lang_apply_rx: mpsc::Receiver<UiLang>,
    vm_windows_reload_tx: mpsc::Sender<VmWindowReloadMsg>,
    vm_windows_reload_rx: mpsc::Receiver<VmWindowReloadMsg>,
}

impl HostAppChannels {
    fn new() -> Self {
        let (persist_apply_tx, persist_apply_rx) = mpsc::channel();
        let (lang_apply_tx, lang_apply_rx) = mpsc::channel();
        let (vm_windows_reload_tx, vm_windows_reload_rx) = mpsc::channel();
        Self {
            persist_apply_tx,
            persist_apply_rx,
            lang_apply_tx,
            lang_apply_rx,
            vm_windows_reload_tx,
            vm_windows_reload_rx,
        }
    }
}

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
    agents: Arc<AgentBindingTable>,
    agent_notice: String,
    announce: HostAnnounceConfig,
    security: ServeSecurity,
    shutdown_rx: watch::Receiver<bool>,
    ui_channels: ServeUiChannels,
) {
    let Some(rt) = host_try_build_serve_runtime() else {
        return;
    };
    let res = rt.block_on(run_serve(
        listen,
        agents,
        agent_notice,
        announce,
        security,
        shutdown_rx,
        ui_channels,
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
        let (persist, env_listen_hint) = Self::host_app_load_persist(cc);
        let host_security = init_host_security();
        let channels = HostAppChannels::new();
        Self::host_app_assemble(
            initial_tray,
            persist,
            env_listen_hint,
            host_security,
            channels,
        )
    }

    fn host_app_load_persist(cc: &eframe::CreationContext<'_>) -> (HostUiPersist, Option<String>) {
        let json_opt = cc.storage.and_then(|s| s.get_string(PERSIST_KEY));
        let mut persist: HostUiPersist = json_opt
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        let mut env_listen_hint = None;
        if let Ok(s) = std::env::var("TITAN_HOST_LISTEN")
            && s.parse::<SocketAddr>().is_ok()
        {
            persist.listen = s;
            env_listen_hint = Some(crate::titan_i18n::hp_env_listen_applied(persist.ui_lang));
        }
        (persist, env_listen_hint)
    }

    #[rustfmt::skip]
    fn host_app_assemble(
        initial_tray: Option<titan_tray::TrayIcon>,
        persist: HostUiPersist,
        env_listen_hint: Option<String>,
        host_security: HostSecurity,
        channels: HostAppChannels,
    ) -> Self {
        // Pure assembly: every field is either threaded from `channels` or initialised with a
        // trivial empty default. Compact form keeps the fn under 30 code lines without a wider
        // HostApp split (cross-cutting `self.foo` rename across many files).
        let HostAppChannels { persist_apply_tx, persist_apply_rx, lang_apply_tx, lang_apply_rx, vm_windows_reload_tx, vm_windows_reload_rx } = channels;
        Self {
            host_security, really_quitting: false, hidden_to_tray: false, _tray: initial_tray, serve_run: None,
            persist_apply_tx: Some(persist_apply_tx), persist_apply_rx,
            lang_apply_tx: Some(lang_apply_tx), lang_apply_rx,
            vm_windows_reload_tx: Some(vm_windows_reload_tx), vm_windows_reload_rx,
            persist, active_tab: 0, status_line: String::new(), env_listen_hint,
            initial_serve_attempted: false, boot_window_focus_once: false,
            settings_open: false, settings_lang_btn_rect: None,
            vm_window_records: Vec::new(), vm_window_masonry_heights: HashMap::new(),
            host_desktop_textures: HashMap::new(), host_resource_stats: HashMap::new(),
            pending_remove_endpoint: None,
        }
    }

    fn start_serve_resolve(
        &mut self,
    ) -> Option<(
        SocketAddr,
        Arc<AgentBindingTable>,
        String,
        HostAnnounceConfig,
    )> {
        let listen = match self.persist.parse_listen() {
            Ok(a) => a,
            Err(e) => {
                self.status_line = e;
                return None;
            }
        };
        let (agents, notice) = HostUiPersist::agent_bindings_for_serve();
        Some((listen, agents, notice, self.persist.to_announce()))
    }

    fn start_serve_spawn_join(
        listen: SocketAddr,
        agents: Arc<AgentBindingTable>,
        agent_notice: String,
        announce: HostAnnounceConfig,
        security: ServeSecurity,
        ui_channels: ServeUiChannels,
    ) -> ServeRun {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let join = std::thread::Builder::new()
            .name("titan-host-serve".into())
            .spawn(move || {
                serve_thread_main(
                    listen,
                    agents,
                    agent_notice,
                    announce,
                    security,
                    shutdown_rx,
                    ui_channels,
                )
            })
            .expect("spawn serve thread");
        ServeRun { shutdown_tx, join }
    }

    pub(crate) fn start_serve(&mut self) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }
        let Some((listen, agents, agent_notice, announce)) = self.start_serve_resolve() else {
            return;
        };
        let security = match self.build_serve_security() {
            Some(s) => s,
            None => return,
        };
        let ui_channels = ServeUiChannels {
            persist_apply_tx: self.persist_apply_tx.clone(),
            lang_apply_tx: self.lang_apply_tx.clone(),
            vm_windows_reload_tx: self.vm_windows_reload_tx.clone(),
        };
        self.serve_run = Some(Self::start_serve_spawn_join(
            listen,
            agents,
            agent_notice,
            announce,
            security,
            ui_channels,
        ));
        self.status_line =
            crate::titan_i18n::hp_control_listening(self.persist.ui_lang, &self.persist.listen);
    }

    fn build_serve_security(&mut self) -> Option<ServeSecurity> {
        Some(build_serve_security(&self.persist, &self.host_security))
    }
}
