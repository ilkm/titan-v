use egui::{Area, Color32, CornerRadius, Frame, Margin, Order, RichText, pos2};

use crate::app::CenterApp;

impl CenterApp {
    pub(crate) fn render_ui_toast(&self, ctx: &egui::Context) {
        let Some(until) = self.ui_toast_until else {
            return;
        };
        let now = ctx.input(|i| i.time);
        if now >= until || self.ui_toast_text.is_empty() {
            return;
        }
        let screen = ctx.screen_rect();
        let p = pos2(screen.center().x - 100.0, screen.max.y - 56.0);
        Area::new(egui::Id::new("titan_center_ui_toast"))
            .order(Order::Foreground)
            .fixed_pos(p)
            .show(ctx, |ui| {
                Frame::NONE
                    .fill(Color32::from_black_alpha(210))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(18, 11))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&self.ui_toast_text)
                                .color(Color32::WHITE)
                                .size(14.0),
                        );
                    });
            });
    }
}
