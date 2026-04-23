//! Host device catalog — YouTube-style vertical tiles (preview on top, meta below).

use egui::{
    Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, Vec2,
};

use super::constants::{ACCENT, HOST_TILE_WIDTH, PANEL_SPACING};
use super::i18n::{host_running_windows_line, t, Msg};
use super::persist_data::HostEndpoint;
use super::widgets::{form_field_row, section_card, subtle_button};
use super::CenterApp;

fn thumbnail_color(key: &str) -> Color32 {
    let mut h: u32 = 2_166_136_261;
    for b in key.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(16_777_619);
    }
    let r = 64 + ((h) & 0x7f) as u8;
    let g = 78 + ((h >> 7) & 0x7f) as u8;
    let b = 108 + ((h >> 14) & 0x6f) as u8;
    Color32::from_rgb(r, g, b)
}

impl CenterApp {
    pub(super) fn panel_hosts(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui_lang;
        section_card(ui, t(lang, Msg::HostsCardTitle), |ui| {
            ui.horizontal(|ui| {
                if subtle_button(ui, t(lang, Msg::BtnAddHost), true).clicked() {
                    self.endpoints.push(HostEndpoint {
                        label: format!("host-{}", self.endpoints.len() + 1),
                        addr: "127.0.0.1:7788".into(),
                        last_caps: String::new(),
                        last_vm_count: 0,
                    });
                }
                if subtle_button(
                    ui,
                    t(lang, Msg::BtnRemoveSelected),
                    !self.endpoints.is_empty(),
                )
                .clicked()
                {
                    let idx = self
                        .selected_host
                        .min(self.endpoints.len().saturating_sub(1));
                    if !self.endpoints.is_empty() {
                        self.endpoints.remove(idx);
                        self.selected_host = self.selected_host.saturating_sub(1);
                    }
                }
            });
            ui.add_space(10.0);

            if self.endpoints.is_empty() {
                ui.add_space(28.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(t(lang, Msg::HostListEmpty))
                            .size(14.0)
                            .italics()
                            .color(ui.visuals().widgets.inactive.text_color()),
                    );
                });
                ui.add_space(20.0);
                return;
            }

            let inner_pad = 8_i8;
            let inner_pad_f = inner_pad as f32;
            let thumb_w = HOST_TILE_WIDTH - 2.0 * inner_pad_f;
            let thumb_h = thumb_w * 9.0 / 16.0;
            let list_h = ui.available_height().clamp(220.0, 520.0);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(list_h)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                        for i in 0..self.endpoints.len() {
                            let is_sel = self.selected_host == i;
                            let (label_s, addr_s, win_n, fill) = {
                                let ep = &self.endpoints[i];
                                (
                                    ep.label.clone(),
                                    ep.addr.clone(),
                                    ep.last_vm_count,
                                    thumbnail_color(&ep.addr),
                                )
                            };

                            let stroke = if is_sel {
                                Stroke::new(1.5, ACCENT.linear_multiply(0.75))
                            } else {
                                Stroke::new(
                                    1.0,
                                    ui.visuals()
                                        .widgets
                                        .noninteractive
                                        .bg_stroke
                                        .color
                                        .linear_multiply(0.5),
                                )
                            };
                            let bg = if is_sel {
                                ACCENT.linear_multiply(0.08)
                            } else {
                                ui.visuals().faint_bg_color.linear_multiply(0.97)
                            };

                            let tile = Frame::NONE
                                .fill(bg)
                                .stroke(stroke)
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::same(inner_pad))
                                .outer_margin(Margin::symmetric(6, 8))
                                .show(ui, |ui| {
                                    ui.set_width(HOST_TILE_WIDTH);
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 6.0;
                                        let (r, _) = ui.allocate_exact_size(
                                            Vec2::new(thumb_w, thumb_h),
                                            Sense::empty(),
                                        );
                                        ui.painter().rect_filled(r, CornerRadius::same(8), fill);
                                        ui.painter().rect_stroke(
                                            r,
                                            CornerRadius::same(8),
                                            Stroke::new(
                                                1.0,
                                                ui.visuals().widgets.noninteractive.bg_stroke.color,
                                            ),
                                            egui::StrokeKind::Inside,
                                        );

                                        ui.with_layout(
                                            Layout::top_down(Align::Min)
                                                .with_cross_align(Align::Min),
                                            |ui| {
                                                ui.spacing_mut().item_spacing.y = 3.0;
                                                ui.label(
                                                    RichText::new(&label_s)
                                                        .strong()
                                                        .size(13.5)
                                                        .color(if is_sel {
                                                            ACCENT
                                                        } else {
                                                            ui.visuals().strong_text_color()
                                                        }),
                                                );
                                                ui.label(
                                                    RichText::new(&addr_s)
                                                        .monospace()
                                                        .size(12.0)
                                                        .color(
                                                            ui.visuals()
                                                                .widgets
                                                                .inactive
                                                                .text_color(),
                                                        ),
                                                );
                                                ui.label(
                                                    RichText::new(host_running_windows_line(
                                                        lang, win_n,
                                                    ))
                                                    .size(12.0)
                                                    .color(
                                                        ui.visuals().widgets.inactive.text_color(),
                                                    ),
                                                );
                                            },
                                        );
                                    });
                                });

                            if tile.response.clicked() {
                                self.selected_host = i;
                                self.control_addr = self.endpoints[i].addr.clone();
                            }
                        }
                    });
                });

            ui.add_space(PANEL_SPACING);
            ui.label(
                RichText::new(t(lang, Msg::HostListSelectedEditTitle))
                    .small()
                    .strong()
                    .color(ACCENT),
            );
            ui.add_space(6.0);
            let i = self
                .selected_host
                .min(self.endpoints.len().saturating_sub(1));
            form_field_row(
                ui,
                RichText::new(t(lang, Msg::HostListNameField)).small(),
                |ui| {
                    if let Some(ep) = self.endpoints.get_mut(i) {
                        ui.add(
                            egui::TextEdit::singleline(&mut ep.label)
                                .desired_width(ui.available_width()),
                        );
                    }
                },
            );
            form_field_row(
                ui,
                RichText::new(t(lang, Msg::HostListAddrField)).small(),
                |ui| {
                    if let Some(ep) = self.endpoints.get_mut(i) {
                        ui.add(
                            egui::TextEdit::singleline(&mut ep.addr)
                                .desired_width(ui.available_width()),
                        );
                    }
                },
            );
        });
    }
}
