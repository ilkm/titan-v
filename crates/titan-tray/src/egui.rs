//! egui integration: wake the winit/eframe loop while the window is hidden so tray + networking stay alive.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use titan_common::UiLang;
use titan_i18n::{Msg, t};

use crate::menu::{self, DesktopProduct};
use tray_icon::menu::{MenuEvent, MenuId};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

static PENDING_MENU_IDS: Mutex<VecDeque<MenuId>> = Mutex::new(VecDeque::new());
static TRAY_HANDLERS_REGISTERED: AtomicBool = AtomicBool::new(false);

fn register_menu_repaint_handler(ctx: &egui::Context) {
    let ctx_menu = ctx.clone();
    MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
        let id = ev.id.clone();
        if let Ok(mut q) = PENDING_MENU_IDS.lock() {
            q.push_back(id);
        }
        ctx_menu.request_repaint();
    }));
}

fn register_tray_icon_show_handler(ctx: &egui::Context, show_id: MenuId) {
    let ctx_tray = ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |ev| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Down,
            ..
        } = ev
        {
            ctx_tray
                .send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
            if let Ok(mut q) = PENDING_MENU_IDS.lock() {
                q.push_back(show_id.clone());
            }
        }
        ctx_tray.request_repaint();
    }));
}

fn register_tray_wakeup_impl(ctx: &egui::Context, product: DesktopProduct) {
    if TRAY_HANDLERS_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    let show_id = product.show_menu_id();
    register_menu_repaint_handler(ctx);
    register_tray_icon_show_handler(ctx, show_id);
}

/// Install global tray/menu hooks for **Titan Center** (same as [`register_tray_wakeup_impl`] with Center ids).
pub fn register_center_tray_wakeup(ctx: &egui::Context) {
    register_tray_wakeup_impl(ctx, DesktopProduct::Center);
}

/// Install global tray/menu hooks for **Titan Host** (distinct menu ids from Center).
pub fn register_host_tray_wakeup(ctx: &egui::Context) {
    register_tray_wakeup_impl(ctx, DesktopProduct::Host);
}

pub fn build_tray_icon(lang: UiLang) -> tray_icon::Result<TrayIcon> {
    build_tray_icon_for(DesktopProduct::Center, lang)
}

pub fn build_host_tray_icon(lang: UiLang) -> tray_icon::Result<TrayIcon> {
    build_tray_icon_for(DesktopProduct::Host, lang)
}

fn build_tray_icon_for(product: DesktopProduct, lang: UiLang) -> tray_icon::Result<TrayIcon> {
    let tooltip = match product {
        DesktopProduct::Center => t(lang, Msg::BrandTitle),
        DesktopProduct::Host => t(lang, Msg::HpWinTitle),
    };
    let m = menu::build_tray_menu(product, lang)
        .map_err(|e| std::io::Error::other(format!("tray menu: {e}")))?;
    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(m))
        .with_menu_on_left_click(false)
        .with_tooltip(tooltip)
        .with_icon(crate::icon::tray_icon_for_lang(product, lang));
    #[cfg(target_os = "macos")]
    let builder = builder.with_icon_as_template(false);
    builder.build()
}

/// Drain tray-queued menu actions (handlers call [`egui::Context::request_repaint`]).
///
/// Returns `true` if **Show main window** was chosen (caller may clear a “hidden to tray” flag).
pub fn poll_tray_for_egui(ctx: &egui::Context, really_quitting: &mut bool) -> bool {
    poll_tray_for_egui_product(ctx, really_quitting, DesktopProduct::Center)
}

pub fn poll_tray_for_egui_product(
    ctx: &egui::Context,
    really_quitting: &mut bool,
    product: DesktopProduct,
) -> bool {
    let show_id = product.show_menu_id();
    let quit_id = product.quit_menu_id();

    let mut showed = false;
    let mut pending = match PENDING_MENU_IDS.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    while let Some(id) = pending.pop_front() {
        if id == show_id {
            showed = true;
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
            ctx.request_repaint();
        } else if id == quit_id {
            *really_quitting = true;
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
        }
    }
    showed
}

/// When the user closes the main window, hide to tray instead of exiting (unless [`really_quitting`]).
///
/// **Note:** With `eframe`, prefer intercepting close in `App::raw_input_hook` (see Titan Center):
/// `eframe` records `close_requested` *before* the hook runs, then checks for [`egui::ViewportCommand::CancelClose`]
/// in the same frame’s output — handling close only inside [`egui::Context::run`] can miss that pairing.
///
/// Returns `true` if the window was just hidden to the tray.
pub fn apply_close_hides_to_tray(ctx: &egui::Context, really_quitting: bool) -> bool {
    if really_quitting {
        return false;
    }
    if ctx.input(|i| i.viewport().close_requested()) {
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Visible(false),
        );
        ctx.request_repaint_after_for(
            std::time::Duration::from_millis(250),
            egui::ViewportId::ROOT,
        );
        return true;
    }
    false
}

/// macOS: use **Regular** activation (Dock + main windows). Call once from the `eframe` creation
/// closure on the main thread before [`build_host_tray_icon`] / [`build_tray_icon`].
#[cfg(all(feature = "egui", target_os = "macos"))]
pub fn macos_ensure_regular_activation_for_egui_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        app.finishLaunching();
        app.activate();
    }
}

/// No-op on non-macOS.
#[cfg(all(feature = "egui", not(target_os = "macos")))]
pub fn macos_ensure_regular_activation_for_egui_app() {}
