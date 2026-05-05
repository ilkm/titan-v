use egui::{Align, Layout, RichText};
use titan_common::{VM_WINDOW_FOLDER_ID_MAX, VM_WINDOW_FOLDER_ID_MIN};

use crate::app::CenterApp;
use crate::app::i18n::{Msg, UiLang, t};
use crate::app::ui::widgets::{
    InsetDropdownLayout, form_field_row, inset_single_select_dropdown, primary_button_large,
    subtle_button,
};

impl CenterApp {
    pub(crate) fn vm_window_create_modal_body(&mut self, ui: &mut egui::Ui, lang: UiLang) {
        let full_w = ui.available_width();
        ui.set_width(full_w);
        self.vm_window_create_clamp_device_ix();
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

    fn vm_window_create_device_row(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_device_dropdown(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_device_trigger_label(&self, lang: UiLang) -> String {
        match self.vm_window_create.device_ix {
            None => t(lang, Msg::CenterWinMgmtDevicePlaceholder).to_string(),
            Some(i) => self
                .endpoints
                .get(i)
                .map(|e| format!("{} — {}", e.label, e.addr))
                .unwrap_or_else(|| t(lang, Msg::CenterWinMgmtDevicePlaceholder).to_string()),
        }
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

    fn vm_window_create_row_cpu(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_row_mem(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_row_disk(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_row_vm_id(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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

    fn vm_window_create_modal_footer(&mut self, ui: &mut egui::Ui, lang: UiLang) {
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
}
