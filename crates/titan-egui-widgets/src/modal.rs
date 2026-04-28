//! Standard [`egui::Window`] shells: opaque centered modals and settings tool window.

use egui::{Align2, Context, Id, Order, Ui, Vec2, WidgetText, Window};

use crate::frames::{opaque_dialog_frame, opaque_dialog_frame_ctx};

/// Source for opaque modal frame paint (parent `Ui` vs root `Context`).
pub enum OpaqueFrameSource<'a> {
    Ui(&'a Ui),
    Ctx(&'a Context),
}

fn opaque_frame_from(src: OpaqueFrameSource<'_>) -> egui::Frame {
    match src {
        OpaqueFrameSource::Ui(ui) => opaque_dialog_frame(ui),
        OpaqueFrameSource::Ctx(ctx) => opaque_dialog_frame_ctx(ctx),
    }
}

/// Centered opaque modal: fixed inner size, first paint at screen center, foreground.
pub fn show_opaque_modal(
    ctx: &Context,
    id: Id,
    title: impl Into<WidgetText>,
    open: &mut bool,
    inner_size: Vec2,
    frame_src: OpaqueFrameSource<'_>,
    add_contents: impl FnOnce(&mut Ui),
) {
    let center_pos = ctx.screen_rect().center() - 0.5 * inner_size;
    Window::new(title)
        .id(id)
        .frame(opaque_frame_from(frame_src))
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fade_in(false)
        .fade_out(false)
        .default_pos(center_pos)
        .fixed_size(inner_size)
        .order(Order::Foreground)
        .show(ctx, add_contents);
}

/// Settings / language popup: same opaque card shell as modals; optional pivot under a screen point.
pub fn show_settings_tool_window(
    ctx: &Context,
    open: &mut bool,
    title: impl Into<WidgetText>,
    anchor_under_btn: Option<egui::Pos2>,
    default_corner_offset: Vec2,
    inner_size: Vec2,
    add_contents: impl FnOnce(&mut Ui),
) {
    let mut w = Window::new(title)
        .id(Id::new("titan_ui_settings_tool_window"))
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fade_in(false)
        .fade_out(false)
        .frame(opaque_dialog_frame_ctx(ctx))
        .fixed_size(inner_size)
        .order(Order::Foreground);
    w = match anchor_under_btn {
        Some(p) => w.pivot(Align2::RIGHT_TOP).fixed_pos(p),
        None => w.default_pos(ctx.screen_rect().right_top() + default_corner_offset),
    };
    w.show(ctx, add_contents);
}
