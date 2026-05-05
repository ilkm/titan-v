//! Center-only **创建窗口** dialog.
//!
//! Center owns SQLite-backed `vm_window_records`. Confirm validates draft against DB,
//! inserts row, then fan-outs authoritative `ApplyVmWindowSnapshot` to target host.

mod draft;
mod panel;

use egui::{Id, Vec2};
use titan_common::{VM_WINDOW_FOLDER_ID_MIN, next_unused_vm_folder_id};

use crate::app::CenterApp;
use crate::app::i18n::{Msg, t};
use crate::app::ui::widgets::{OpaqueFrameSource, show_opaque_modal};
use crate::app::vm_window_push_to_hosts;

pub(crate) use draft::CenterVmWindowCreateForm;
use draft::{clamp_device_ix, vm_window_create_prepare_row, vm_window_local_persist_create};

const CREATE_DLG_INNER: Vec2 = Vec2::new(480.0, 420.0);

impl CenterApp {
    fn vm_window_create_refresh_vm_id_for_device(&mut self, dev_ix: usize) {
        let Some(mut ep) = self.endpoints.get(dev_ix).cloned() else {
            return;
        };
        ep.ensure_device_id();
        let did = ep.device_id;
        let existing = self
            .vm_window_records
            .iter()
            .filter(|r| r.device_id == did)
            .map(|r| r.vm_id);
        self.vm_window_create.vm_id = next_unused_vm_folder_id(existing);
    }

    pub(crate) fn vm_window_create_on_device_selected(&mut self, ix: usize) {
        self.vm_window_create.device_ix = Some(ix);
        self.vm_window_create_refresh_vm_id_for_device(ix);
    }

    pub(crate) fn open_vm_window_create_dialog(&mut self) {
        self.vm_window_create.inline_err.clear();
        self.vm_window_create.dialog_open = true;
    }

    pub(crate) fn render_vm_window_create_dialog(&mut self, ctx: &egui::Context) {
        if !self.vm_window_create.dialog_open {
            return;
        }
        let mut open = self.vm_window_create.dialog_open;
        let lang = self.ui_lang;
        let title = t(lang, Msg::HpWinMgmtDialogTitle);
        show_opaque_modal(
            ctx,
            Id::new("titan_center_vm_window_create"),
            title,
            &mut open,
            CREATE_DLG_INNER,
            OpaqueFrameSource::Ctx(ctx),
            |ui| self.vm_window_create_modal_body(ui, lang),
        );
        self.vm_window_create.dialog_open = open && self.vm_window_create.dialog_open;
    }

    fn try_submit_vm_window_create(&mut self, lang: crate::app::i18n::UiLang) {
        self.vm_window_create.inline_err.clear();
        let row = match vm_window_create_prepare_row(
            &self.endpoints,
            &self.vm_window_create,
            &mut self.vm_window_create_id_nonce,
            lang,
        ) {
            Ok(r) => r,
            Err(msg) => {
                self.vm_window_create.inline_err = msg.to_string();
                return;
            }
        };
        if let Err(msg) = vm_window_local_persist_create(&row, lang) {
            self.vm_window_create.inline_err = msg;
            return;
        }
        let did = row.device_id.clone();
        self.vm_window_records.push(row);
        vm_window_push_to_hosts::push_snapshot_for_device(
            &self.endpoints,
            &self.vm_window_records,
            &did,
        );
        self.vm_window_create_apply_success(lang);
    }

    fn vm_window_create_apply_success(&mut self, lang: crate::app::i18n::UiLang) {
        self.vm_window_create.dialog_open = false;
        self.vm_window_create.device_ix = None;
        self.vm_window_create.vm_id = VM_WINDOW_FOLDER_ID_MIN;
        self.vm_window_create.inline_err.clear();
        let now = self.ctx.input(|i| i.time);
        self.ui_toast_text = t(lang, Msg::CenterWinMgmtToastCreated).to_string();
        self.ui_toast_until = Some(now + 4.0);
    }

    pub(crate) fn vm_window_create_clamp_device_ix(&mut self) {
        clamp_device_ix(&mut self.vm_window_create, self.endpoints.len());
    }
}
