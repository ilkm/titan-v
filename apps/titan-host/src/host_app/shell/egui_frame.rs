use eframe::egui;
use titan_common::VmWindowRecord;

use crate::serve::VmWindowReloadMsg;
use crate::titan_i18n::{self as i18n, Msg};

use crate::host_app::model::{HostApp, PERSIST_KEY};

impl HostApp {
    fn boot_focus_once_if_needed(&mut self, ctx: &egui::Context) {
        if !self.boot_window_focus_once && !self.hidden_to_tray {
            self.boot_window_focus_once = true;
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
        }
    }

    fn sync_tray_wakeup_and_repaint(&mut self, ctx: &egui::Context) {
        if titan_tray::poll_tray_for_egui_product(
            ctx,
            &mut self.really_quitting,
            titan_tray::DesktopProduct::Host,
        ) {
            self.hidden_to_tray = false;
        }
        if self.hidden_to_tray {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }
    }

    fn show_host_chrome(&mut self, ctx: &egui::Context) {
        let title = i18n::t(self.persist.ui_lang, Msg::HpWinTitle).to_string();
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Title(title));
        self.render_host_top_panel(ctx);
        self.render_host_side_nav(ctx);
        self.render_host_central_panel(ctx);
    }

    fn replace_vm_window_rows(&mut self, records: Vec<VmWindowRecord>) {
        self.vm_window_records = records;
        self.vm_window_masonry_heights.retain(|k, _| {
            self.vm_window_records
                .iter()
                .any(|r| r.record_id.as_str() == k.as_str())
        });
    }

    fn drain_vm_windows_reload(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.vm_windows_reload_rx.try_recv() {
            match msg {
                VmWindowReloadMsg::Replace { records } => self.replace_vm_window_rows(records),
            }
            ctx.request_repaint();
        }
    }
}

impl eframe::App for HostApp {
    /// Do not persist egui memory: otherwise a previous "close to tray" session restores
    /// `ViewportCommand::Visible(false)` and the main window can stay hidden on launch.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        match serde_json::to_string(&self.persist) {
            Ok(json) => storage.set_string(PERSIST_KEY, json),
            Err(e) => tracing::warn!(error = %e, "host persist: serialize failed"),
        }
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.really_quitting || raw_input.viewport_id != egui::ViewportId::ROOT {
            return;
        }
        if !raw_input.viewport().close_requested() {
            return;
        }
        #[cfg(windows)]
        if titan_tray::consume_windows_tray_quit_close_request() {
            self.really_quitting = true;
            return;
        }
        if let Some(vp) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            vp.events.retain(|e| *e != egui::ViewportEvent::Close);
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        titan_tray::hide_main_window_to_tray(ctx);
        self.hidden_to_tray = true;
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(next) = self.persist_apply_rx.try_recv() {
            self.persist = next;
            self.start_serve();
        }
        while let Ok(lang) = self.lang_apply_rx.try_recv() {
            self.persist.ui_lang = lang;
            ctx.request_repaint();
        }
        self.drain_vm_windows_reload(ctx);
        if let Some(tray) = self._tray.as_ref() {
            titan_tray::sync_tray_if_needed(
                tray,
                titan_tray::DesktopProduct::Host,
                self.persist.ui_lang,
            );
        }
        self.boot_focus_once_if_needed(ctx);
        self.sync_tray_wakeup_and_repaint(ctx);
        if !self.initial_serve_attempted {
            self.initial_serve_attempted = true;
            self.start_serve();
        }
        self.show_host_chrome(ctx);
    }
}
