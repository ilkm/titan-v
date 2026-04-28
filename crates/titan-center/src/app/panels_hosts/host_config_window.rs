//! Host UI JSON draft: floating window from device card preview (SQLite + push over control plane).

use egui::{Align, Layout, RichText, Vec2};

use titan_common::ControlRequest;

use super::super::device_store;
use super::super::i18n::{t, Msg, UiLang};
use super::super::net_client::exchange_one;
use super::super::net_msg::NetUiMsg;
use super::super::widgets::{
    multiline_inset, primary_button_large, show_opaque_modal, subtle_button_large,
    OpaqueFrameSource,
};
use super::super::CenterApp;
use super::helpers::ADD_HOST_DLG_MUTED;

fn host_ui_push_exchange(addr: &str, json: String) -> (bool, String) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => return (false, format!("runtime: {e}")),
    };
    let res = rt.block_on(exchange_one(
        addr,
        &ControlRequest::ApplyHostUiPersistJson { json },
    ));
    match res {
        Ok(titan_common::ControlResponse::HostUiPersistAck { ok, detail }) => (ok, detail),
        Ok(titan_common::ControlResponse::ServerError { message, .. }) => (false, message),
        Ok(other) => (false, format!("unexpected response: {other:?}")),
        Err(e) => (false, e.to_string()),
    }
}

fn host_config_win_status(ui: &mut egui::Ui, msg: &str) {
    if msg.is_empty() {
        return;
    }
    ui.label(
        RichText::new(msg)
            .size(12.5)
            .line_height(Some(18.0))
            .color(ADD_HOST_DLG_MUTED),
    );
    ui.add_space(12.0);
}

fn host_config_win_subtitle(ui: &mut egui::Ui, addr: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(addr)
                .monospace()
                .size(12.5)
                .line_height(Some(18.0))
                .color(ADD_HOST_DLG_MUTED),
        )
        .wrap(),
    );
    ui.add_space(16.0);
}

fn host_config_win_load_save_row(app: &mut CenterApp, ui: &mut egui::Ui, lang: UiLang, idx: usize) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        if subtle_button_large(ui, t(lang, Msg::HostConfigWinLoadDb), true).clicked() {
            app.host_managed_load_selected(idx);
        }
        if subtle_button_large(ui, t(lang, Msg::HostConfigWinSaveDb), true).clicked() {
            app.host_managed_save_selected(idx);
        }
    });
}

fn host_config_win_json_block(app: &mut CenterApp, ui: &mut egui::Ui) {
    const MIN_H: f32 = 220.0;
    multiline_inset(ui, MIN_H, &mut app.host_managed_draft_json, "");
}

fn host_config_win_footer(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    addr: &str,
    close: &mut bool,
) {
    let full_w = ui.available_width();
    let can_push = !app.host_managed_draft_json.trim().is_empty();
    ui.allocate_ui_with_layout(
        egui::vec2(full_w, 48.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            if primary_button_large(ui, t(lang, Msg::HostConfigWinPushHost), can_push).clicked() {
                app.host_managed_spawn_push(addr.to_string(), app.host_managed_draft_json.clone());
            }
            if subtle_button_large(ui, t(lang, Msg::HostConfigWinClose), true).clicked() {
                *close = true;
            }
        },
    );
}

fn host_config_win_body(
    app: &mut CenterApp,
    ui: &mut egui::Ui,
    lang: UiLang,
    idx: usize,
    addr: &str,
    close: &mut bool,
) {
    let full_w = ui.available_width();
    ui.set_width(full_w);
    ui.spacing_mut().item_spacing.y = 0.0;
    host_config_win_status(ui, &app.host_managed_last_msg);
    host_config_win_subtitle(ui, addr);
    host_config_win_load_save_row(app, ui, lang, idx);
    ui.add_space(12.0);
    host_config_win_json_block(app, ui);
    ui.add_space(20.0);
    host_config_win_footer(app, ui, lang, addr, close);
}

impl CenterApp {
    /// Online card preview → Configure: select host, open window, load SQLite draft.
    pub(crate) fn open_host_config_from_card(&mut self, card_index: usize) {
        self.selected_host = card_index;
        self.host_config_window_open = true;
        self.host_managed_load_selected(card_index);
    }

    fn host_managed_load_selected(&mut self, idx: usize) {
        let ep = match self.endpoints.get(idx) {
            Some(e) => e.clone(),
            None => return,
        };
        let db = device_store::registration_db_path();
        let mut d = ep;
        d.ensure_device_id();
        self.host_managed_last_msg = match device_store::load_host_managed_config(&db, &d.device_id)
        {
            Ok(Some(j)) => {
                self.host_managed_draft_json = j;
                "Loaded draft from SQLite.".into()
            }
            Ok(None) => "No draft row for this device_id.".into(),
            Err(e) => format!("SQLite: {e}"),
        };
    }

    fn host_managed_save_selected(&mut self, idx: usize) {
        let ep = match self.endpoints.get(idx) {
            Some(e) => e.clone(),
            None => return,
        };
        let db = device_store::registration_db_path();
        let mut d = ep;
        d.ensure_device_id();
        self.host_managed_last_msg = match device_store::upsert_host_managed_config(
            &db,
            &d.device_id,
            &self.host_managed_draft_json,
        ) {
            Ok(()) => "Saved draft to SQLite.".into(),
            Err(e) => format!("SQLite: {e}"),
        };
    }

    fn host_managed_spawn_push(&self, addr: String, json: String) {
        let tx = self.net_tx.clone();
        let _ = std::thread::Builder::new()
            .name("titan-center-host-ui-push".into())
            .spawn(move || {
                let (ok, detail) = host_ui_push_exchange(&addr, json);
                let _ = tx.send(NetUiMsg::HostUiPushDone { ok, detail });
            });
    }

    fn host_config_window_show(
        &mut self,
        ctx: &egui::Context,
        title: &str,
        idx: usize,
        addr: &str,
        win_open: &mut bool,
        close: &mut bool,
    ) {
        const DIALOG_INNER: Vec2 = Vec2::new(520.0, 440.0);
        let lang = self.ui_lang;
        show_opaque_modal(
            ctx,
            egui::Id::new("titan_center_host_config_window"),
            title,
            win_open,
            DIALOG_INNER,
            OpaqueFrameSource::Ctx(ctx),
            |ui| {
                host_config_win_body(self, ui, lang, idx, addr, close);
            },
        );
    }

    pub(crate) fn render_host_config_window(&mut self, ctx: &egui::Context) {
        if !self.host_config_window_open || self.endpoints.is_empty() {
            return;
        }
        let title = t(self.ui_lang, Msg::HostConfigWinTitle);
        let idx = self
            .selected_host
            .min(self.endpoints.len().saturating_sub(1));
        let addr = self.endpoints[idx].addr.clone();
        let mut close = false;
        let mut win_open = self.host_config_window_open;
        self.host_config_window_show(ctx, title, idx, &addr, &mut win_open, &mut close);
        self.host_config_window_open = win_open && !close;
    }
}
