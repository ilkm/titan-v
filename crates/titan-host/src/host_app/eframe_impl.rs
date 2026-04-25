use eframe::egui;

use super::model::HostApp;

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
        egui::TopBottomPanel::top("host_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Titan Host");
                ui.separator();
                if ui.selectable_label(self.active_tab == 0, "服务").clicked() {
                    self.active_tab = 0;
                }
                if ui
                    .selectable_label(self.active_tab == 1, "批量创建")
                    .clicked()
                {
                    self.active_tab = 1;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            0 => self.panel_service(ui),
            1 => self.panel_batch(ui),
            _ => {}
        });
    }
}

impl eframe::App for HostApp {
    /// Do not persist egui memory: otherwise a previous "close to tray" session restores
    /// `ViewportCommand::Visible(false)` and the main window can stay hidden on launch.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.really_quitting || raw_input.viewport_id != egui::ViewportId::ROOT {
            return;
        }
        if !raw_input.viewport().close_requested() {
            return;
        }
        if let Some(vp) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            vp.events.retain(|e| *e != egui::ViewportEvent::Close);
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Visible(false),
        );
        ctx.request_repaint_after_for(
            std::time::Duration::from_millis(250),
            egui::ViewportId::ROOT,
        );
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
        self.boot_focus_once_if_needed(ctx);
        self.sync_tray_wakeup_and_repaint(ctx);
        self.drain_provision_log();
        if !self.initial_serve_attempted {
            self.initial_serve_attempted = true;
            self.start_serve();
        }
        self.show_host_chrome(ctx);
    }
}
