//! Status board, accounts/proxies, virtual slot grid, activity.

use egui::{CornerRadius, RichText, Sense, Stroke};

use super::constants::VIRTUAL_SLOTS;
use super::i18n::{
    fmt_slot_grid_header, fmt_slot_line_empty, fmt_slot_line_vm, fmt_status_board_stats, t, Msg,
};
use super::widgets::section_card;
use super::CenterApp;

impl CenterApp {
    /// Placeholder preview region until streaming is wired (window grid 1–40 per host).
    pub(super) fn panel_window_preview_placeholder(&self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::WindowPreviewTitle), |ui| {
            ui.label(
                RichText::new(t(self.ui_lang, Msg::WindowPreviewHint))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(8.0);
            let h = 140.0;
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), h), Sense::empty());
            let fill = ui
                .visuals()
                .widgets
                .noninteractive
                .bg_fill
                .linear_multiply(1.08);
            let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
            ui.painter().rect(
                rect,
                CornerRadius::same(8),
                fill,
                stroke,
                egui::StrokeKind::Inside,
            );
        });
    }

    pub(super) fn panel_status_board(&self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::StatusBoardTitle), |ui| {
            ui.label(
                RichText::new(fmt_status_board_stats(
                    self.ui_lang,
                    self.vm_inventory.len(),
                    self.accounts.len(),
                    self.proxy_labels.len(),
                ))
                .small()
                .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for row in self.vm_inventory.iter().take(24) {
                        ui.label(
                            RichText::new(format!("{} · {:?}", row.name, row.state))
                                .small()
                                .monospace(),
                        );
                    }
                });
        });
    }

    pub(super) fn panel_datacenter_stores(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::AccountsCardTitle), |ui| {
            ui.horizontal(|ui| {
                if super::widgets::subtle_button(ui, t(self.ui_lang, Msg::BtnAccount), true)
                    .clicked()
                {
                    self.accounts
                        .push(format!("account-{}", self.accounts.len() + 1));
                }
                if super::widgets::subtle_button(ui, t(self.ui_lang, Msg::BtnProxyLabel), true)
                    .clicked()
                {
                    self.proxy_labels
                        .push(format!("proxy-{}", self.proxy_labels.len() + 1));
                }
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::AccountsLabel))
                    .small()
                    .strong(),
            );
            ui.label(RichText::new(self.accounts.join(", ")).small());
            ui.add_space(4.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::ProxiesLabel))
                    .small()
                    .strong(),
            );
            ui.label(RichText::new(self.proxy_labels.join(", ")).small());
            ui.add_space(8.0);
            ui.label(
                RichText::new(t(self.ui_lang, Msg::ScriptArtifactHint))
                    .small()
                    .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.last_script_version)
                    .desired_width(f32::INFINITY)
                    .hint_text(t(self.ui_lang, Msg::HintScriptVersion)),
            );
        });
    }

    pub(super) fn panel_virtual_slots(&self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::SlotGridTitle), |ui| {
            let host_label = self
                .endpoints
                .get(self.selected_host)
                .map(|e| e.label.as_str())
                .unwrap_or(t(self.ui_lang, Msg::NoHost));
            ui.label(
                RichText::new(fmt_slot_grid_header(
                    self.ui_lang,
                    VIRTUAL_SLOTS,
                    host_label,
                ))
                .small()
                .color(ui.visuals().widgets.inactive.text_color()),
            );
            ui.add_space(6.0);
            let row_h = 18.0;
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .auto_shrink([false, false])
                .show_rows(ui, row_h, VIRTUAL_SLOTS, |ui, range| {
                    for i in range {
                        let label = self.vm_inventory.get(i).map_or_else(
                            || fmt_slot_line_empty(self.ui_lang, host_label, i),
                            |v| {
                                fmt_slot_line_vm(
                                    self.ui_lang,
                                    host_label,
                                    i,
                                    &v.name,
                                    &format!("{:?}", v.state),
                                )
                            },
                        );
                        ui.label(RichText::new(label).small().monospace());
                    }
                });
        });
    }

    pub(super) fn panel_tasks(&mut self, ui: &mut egui::Ui) {
        section_card(ui, t(self.ui_lang, Msg::ActivityTitle), |ui| {
            if !self.last_action.is_empty() {
                ui.label(
                    RichText::new(&self.last_action)
                        .small()
                        .line_height(Some(18.0)),
                );
            } else {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::ActivityHint))
                        .small()
                        .italics()
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
            }
        });
    }
}
