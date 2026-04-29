use std::net::SocketAddr;

use titan_common::UiLang;
use tokio::sync::watch;

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
}
