//! VM inventory / window management (per-VM tiles).

use egui::{CornerRadius, RichText, Sense, Stroke};

use super::constants::{CONTENT_COLUMN_GAP, HOST_TILE_MIN_WIDTH, PANEL_SPACING};
use super::i18n::{t, Msg};
use super::widgets::section_card;
use super::CenterApp;

impl CenterApp {
    /// Window management: VM-centric tiles; empty inventory → only 暂无数据.
    pub(super) fn panel_window_management(&mut self, ui: &mut egui::Ui) {
        if self.inventory_slice().is_empty() {
            ui.add_space(48.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::NoDataShort))
                        .size(15.0)
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
            });
            return;
        }

        let inner = ui.available_width();
        let gap = CONTENT_COLUMN_GAP;
        let left_w = (inner * 0.42).clamp(200.0, (inner - gap - 160.0).max(200.0));
        let right_w = (inner - gap - left_w).max(160.0);

        ui.horizontal(|ui| {
            ui.set_min_width(inner);
            ui.vertical(|ui| {
                ui.set_width(left_w);
                section_card(ui, t(self.ui_lang, Msg::VmInventoryTitle), |ui| {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("vm_inventory_youtube_cards")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.panel_vm_youtube_cards(ui);
                        });
                });
            });
            ui.add_space(gap);
            ui.vertical(|ui| {
                ui.set_width(right_w);
                self.panel_window_preview_placeholder(ui);
                ui.add_space(PANEL_SPACING);
                self.panel_host_disk_volumes(ui);
                ui.add_space(PANEL_SPACING);
                self.panel_virtual_slots(ui);
            });
        });
    }

    fn vm_thumb_color(name: &str) -> egui::Color32 {
        let mut h: u32 = 2166136261;
        for b in name.as_bytes() {
            h ^= *b as u32;
            h = h.wrapping_mul(16777619);
        }
        let r = 90 + (h & 0x3f) as u8;
        let g = 90 + ((h >> 8) & 0x3f) as u8;
        let b = 90 + ((h >> 16) & 0x3f) as u8;
        egui::Color32::from_rgb(r, g, b)
    }

    fn panel_vm_youtube_cards(&self, ui: &mut egui::Ui) {
        let host_line = self
            .endpoints
            .get(self.selected_host)
            .map(|ep| format!("{} · {}", t(self.ui_lang, Msg::VmTileHostPrefix), ep.label))
            .unwrap_or_else(|| t(self.ui_lang, Msg::NoHost).to_string());

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.spacing_mut().item_spacing.y = 12.0;

            for row in self.inventory_slice() {
                let w = ui.available_width().clamp(HOST_TILE_MIN_WIDTH, 280.0);
                ui.vertical(|ui| {
                    ui.set_width(w);
                    let thumb_h = (w * 9.0 / 16.0).max(72.0);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, thumb_h), Sense::empty());
                    let fill = Self::vm_thumb_color(&row.name).linear_multiply(0.55);
                    let stroke =
                        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(8),
                        fill,
                        stroke,
                        egui::StrokeKind::Inside,
                    );

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(&row.name)
                            .strong()
                            .size(13.0)
                            .color(ui.visuals().strong_text_color()),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&host_line)
                            .small()
                            .color(ui.visuals().widgets.inactive.text_color()),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!(
                            "{} · {:?}",
                            t(self.ui_lang, Msg::ColState),
                            row.state
                        ))
                        .small()
                        .color(ui.visuals().widgets.inactive.text_color()),
                    );
                });
                ui.add_space(4.0);
            }
        });
    }

    fn panel_host_disk_volumes(&self, ui: &mut egui::Ui) {
        section_card(ui, "Host disk volumes (telemetry)", |ui| {
            if self.disk_volumes_slice().is_empty() {
                ui.label(
                    RichText::new(t(self.ui_lang, Msg::NoDataShort))
                        .small()
                        .color(ui.visuals().widgets.inactive.text_color()),
                );
                return;
            }
            egui::ScrollArea::vertical()
                .id_salt("host_disk_volumes_list")
                .max_height(200.0)
                .show(ui, |ui| {
                    for v in self.disk_volumes_slice() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&v.mount)
                                    .monospace()
                                    .size(12.0)
                                    .color(ui.visuals().text_color()),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let free_gib = v.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                                    let tot_gib = v.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                                    ui.label(
                                        RichText::new(format!("{free_gib:.1} / {tot_gib:.1} GiB"))
                                            .small()
                                            .color(ui.visuals().widgets.inactive.text_color()),
                                    );
                                },
                            );
                        });
                    }
                });
        });
    }
}
