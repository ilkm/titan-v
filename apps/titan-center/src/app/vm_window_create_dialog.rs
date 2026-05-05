//! Center-only **创建窗口** dialog.
//!
//! Center is the sole owner of the SQLite-backed `vm_window_records`. Confirming this dialog
//! validates the draft against the local DB, inserts the row, then fan-outs an authoritative
//! [`titan_common::ControlRequest::ApplyVmWindowSnapshot`] to the affected host
//! (so the host UI redraws in real time without polling).

use egui::{Align, Id, Layout, RichText, Vec2};

use titan_common::{
    VM_WINDOW_FOLDER_ID_MAX, VM_WINDOW_FOLDER_ID_MIN, VmWindowRecord, next_unused_vm_folder_id,
};

use crate::app::CenterApp;
use crate::app::i18n::{Msg, t};
use crate::app::persist_data::HostEndpoint;
use crate::app::ui::widgets::{
    InsetDropdownLayout, OpaqueFrameSource, form_field_row, inset_single_select_dropdown,
    primary_button_large, show_opaque_modal, subtle_button,
};
use crate::app::vm_window_db;
use crate::app::vm_window_push_to_hosts;

const CREATE_DLG_INNER: Vec2 = Vec2::new(480.0, 420.0);

/// Draft for the center **创建窗口** modal.
///
/// `vm_directory` is no longer user-supplied: the host computes the absolute path under
/// its own `vm_root_directory` from `{vm_id}` when applying the row, so the center stores
/// the relative folder name only.
#[derive(Debug, Clone)]
pub(crate) struct CenterVmWindowCreateForm {
    pub dialog_open: bool,
    pub device_ix: Option<usize>,
    pub cpu_count: u32,
    pub memory_mib: u32,
    pub disk_mib: u32,
    pub vm_id: u32,
    pub inline_err: String,
}

impl CenterVmWindowCreateForm {
    pub(crate) fn with_defaults() -> Self {
        Self {
            dialog_open: false,
            device_ix: None,
            cpu_count: 2,
            memory_mib: 4096,
            disk_mib: 131_072,
            vm_id: VM_WINDOW_FOLDER_ID_MIN,
            inline_err: String::new(),
        }
    }
}

fn unix_millis_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn clamp_device_ix(form: &mut CenterVmWindowCreateForm, n: usize) {
    if let Some(ix) = form.device_ix
        && ix >= n
    {
        form.device_ix = None;
    }
}

fn vm_window_create_resolve_dev_ix(
    endpoints: &[HostEndpoint],
    device_ix: Option<usize>,
    lang: crate::app::i18n::UiLang,
) -> Result<usize, &'static str> {
    if endpoints.is_empty() {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevices));
    }
    let Some(ix) = device_ix else {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevice));
    };
    if ix >= endpoints.len() {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevice));
    }
    Ok(ix)
}

fn vm_window_create_build_row(
    ep: &mut HostEndpoint,
    form: &CenterVmWindowCreateForm,
    nonce: &mut u64,
) -> VmWindowRecord {
    *nonce = nonce.wrapping_add(1);
    ep.ensure_device_id();
    let ms = unix_millis_now();
    VmWindowRecord {
        record_id: format!("cw-{ms}-{}", *nonce),
        device_id: ep.device_id.clone(),
        host_control_addr: ep.addr.clone(),
        host_label: ep.label.clone(),
        cpu_count: form.cpu_count,
        memory_mib: form.memory_mib,
        disk_mib: form.disk_mib,
        vm_directory: form.vm_id.to_string(),
        vm_id: form.vm_id,
        created_at_unix_ms: ms,
    }
}

fn vm_window_create_prepare_row(
    endpoints: &[HostEndpoint],
    form: &CenterVmWindowCreateForm,
    nonce: &mut u64,
    lang: crate::app::i18n::UiLang,
) -> Result<VmWindowRecord, &'static str> {
    let dev_ix = vm_window_create_resolve_dev_ix(endpoints, form.device_ix, lang)?;
    if !(VM_WINDOW_FOLDER_ID_MIN..=VM_WINDOW_FOLDER_ID_MAX).contains(&form.vm_id) {
        return Err(t(lang, Msg::HpWinMgmtErrVmId));
    }
    let mut ep = endpoints[dev_ix].clone();
    Ok(vm_window_create_build_row(&mut ep, form, nonce))
}

fn vm_window_local_persist_create(
    row: &VmWindowRecord,
    lang: crate::app::i18n::UiLang,
) -> Result<(), String> {
    let path = vm_window_db::center_vm_window_db_path();
    match vm_window_db::conflicts_for(&path, &row.record_id, row.vm_id, &row.vm_directory) {
        Ok(true) => return Err(t(lang, Msg::HpWinMgmtErrVmId).to_string()),
        Ok(false) => {}
        Err(e) => {
            tracing::warn!(error = %e, "vm_window_db: conflicts_for");
            return Err(t(lang, Msg::CenterWinMgmtHostSyncErr).to_string());
        }
    }
    if let Err(e) = vm_window_db::upsert(&path, row) {
        tracing::warn!(error = %e, "vm_window_db: upsert");
        return Err(t(lang, Msg::CenterWinMgmtHostSyncErr).to_string());
    }
    Ok(())
}

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

    fn vm_window_create_on_device_selected(&mut self, ix: usize) {
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
            |ui| {
                self.vm_window_create_modal_body(ui, lang);
            },
        );
        // `open` only tracks the Window chrome (e.g. ✕); cancel/success set `dialog_open` on `self`.
        // Do not overwrite those with the stale local `open` (would keep the modal stuck open).
        self.vm_window_create.dialog_open = open && self.vm_window_create.dialog_open;
    }

    fn vm_window_create_modal_body(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        let full_w = ui.available_width();
        ui.set_width(full_w);
        clamp_device_ix(&mut self.vm_window_create, self.endpoints.len());
        self.vm_window_create_error_banner(ui);
        self.vm_window_create_device_row(ui, lang);
        self.vm_window_create_row_cpu(ui, lang);
        self.vm_window_create_row_mem(ui, lang);
        self.vm_window_create_row_disk(ui, lang);
        self.vm_window_create_row_vm_id(ui, lang);
        ui.add_space(12.0);
        self.vm_window_create_modal_footer(ui, lang);
    }

    fn vm_window_create_error_banner(&mut self, ui: &mut egui::Ui) {
        if self.vm_window_create.inline_err.is_empty() {
            return;
        }
        let err = ui.visuals().error_fg_color;
        ui.label(
            RichText::new(&self.vm_window_create.inline_err)
                .small()
                .color(err),
        );
        ui.add_space(6.0);
    }

    fn vm_window_create_device_row(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        if self.endpoints.is_empty() {
            form_field_row(
                ui,
                RichText::new(t(lang, Msg::CenterWinMgmtDevice)).small(),
                |ui| {
                    ui.weak(t(lang, Msg::CenterWinMgmtErrNoDevices));
                },
            );
            return;
        }
        self.vm_window_create_device_dropdown(ui, lang);
    }

    fn vm_window_create_device_trigger_label(&self, lang: crate::app::i18n::UiLang) -> String {
        match self.vm_window_create.device_ix {
            None => t(lang, Msg::CenterWinMgmtDevicePlaceholder).to_string(),
            Some(i) => self
                .endpoints
                .get(i)
                .map(|e| format!("{} — {}", e.label, e.addr))
                .unwrap_or_else(|| t(lang, Msg::CenterWinMgmtDevicePlaceholder).to_string()),
        }
    }

    fn vm_window_create_device_dropdown(
        &mut self,
        ui: &mut egui::Ui,
        lang: crate::app::i18n::UiLang,
    ) {
        let trigger = self.vm_window_create_device_trigger_label(lang);
        let w = ui.available_width();
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::CenterWinMgmtDevice)).small(),
            |ui| {
                inset_single_select_dropdown(
                    ui,
                    "center_vm_window_create_device",
                    w,
                    trigger,
                    220.0,
                    InsetDropdownLayout::default(),
                    |ui| self.vm_window_create_device_menu_rows(ui),
                );
            },
        );
    }

    fn vm_window_create_device_menu_rows(&mut self, ui: &mut egui::Ui) {
        let labels: Vec<String> = self
            .endpoints
            .iter()
            .map(|ep| format!("{} — {}", ep.label, ep.addr))
            .collect();
        for (i, row) in labels.into_iter().enumerate() {
            let sel = self.vm_window_create.device_ix == Some(i);
            if ui.selectable_label(sel, row).clicked() {
                self.vm_window_create_on_device_selected(i);
            }
        }
    }

    fn vm_window_create_row_cpu(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HpWinMgmtCpu)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.vm_window_create.cpu_count)
                        .speed(0.25)
                        .range(1..=256),
                );
            },
        );
    }

    fn vm_window_create_row_mem(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HpWinMgmtMem)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.vm_window_create.memory_mib)
                        .speed(64.0)
                        .range(256..=1_048_576),
                );
            },
        );
    }

    fn vm_window_create_row_disk(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HpWinMgmtDisk)).small(),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.vm_window_create.disk_mib)
                        .speed(1024.0)
                        .range(1024..=16_777_216),
                );
            },
        );
    }

    fn vm_window_create_row_vm_id(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        form_field_row(
            ui,
            RichText::new(t(lang, Msg::HpWinMgmtVmId)).small(),
            |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.vm_window_create.vm_id)
                            .speed(1.0)
                            .range(VM_WINDOW_FOLDER_ID_MIN..=VM_WINDOW_FOLDER_ID_MAX),
                    );
                    ui.label(
                        RichText::new(t(lang, Msg::HpWinMgmtVmIdHint))
                            .small()
                            .weak(),
                    );
                });
            },
        );
    }

    fn vm_window_create_modal_footer(&mut self, ui: &mut egui::Ui, lang: crate::app::i18n::UiLang) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button_large(ui, t(lang, Msg::HpWinMgmtConfirm), true).clicked() {
                    self.try_submit_vm_window_create(lang);
                }
                if subtle_button(ui, t(lang, Msg::BtnCancel), true).clicked() {
                    self.vm_window_create.inline_err.clear();
                    self.vm_window_create.dialog_open = false;
                }
            });
        });
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
}
