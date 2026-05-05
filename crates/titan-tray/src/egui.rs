//! egui integration: wake the winit/eframe loop while the window is hidden so tray + networking stay alive.
//!
//! **Windows / Linux (not macOS):** tray and `muda` menus run inside nested OS message loops. Calling
//! [`egui::Context::request_repaint`] there on the UI thread may not wake winit / egui reliably; egui’s
//! integration wakes the loop when [`request_repaint`](egui::Context::request_repaint) runs from
//! **another** thread (see egui `Context` docs). We reschedule repaints off the UI thread.
//!
//! **Windows — critical:** after `ViewportCommand::Visible(false)`, eframe can no longer process
//! `Visible(true)` because hidden windows stop receiving `WM_PAINT` / `RedrawRequested`, which is the
//! only place eframe drains viewport commands (egui issues #5229 / #3655). We therefore bypass the
//! viewport command queue for show / hide and call `ShowWindow` / `SetForegroundWindow` directly on
//! the root `HWND` stored by [`set_windows_tray_wake_hwnd`]. Desktop apps must call it once from the
//! `eframe` creation closure and use [`hide_main_window_to_tray`] instead of sending `Visible(false)`.
//!
//! **Windows tray Quit:** [`egui::ViewportCommand::Close`] can also stall when the window is hidden.
//! Microsoft’s pattern for exiting from a notification icon is to post `WM_CLOSE` to the owning
//! top-level `HWND` (see Win32 *NotificationIcon* / *Window Procedures*). We set a one-shot flag and
//! `PostMessageW(WM_CLOSE)`; the app’s [`eframe::App::raw_input_hook`] must call
//! [`consume_windows_tray_quit_close_request`] when `close_requested` is set so the close is **not**
//! converted into “hide to tray” (same frame as [`CancelClose`]).
//!
//! **Hidden window can’t receive `WM_PAINT`** — MSDN *WM_PAINT* remark and longstanding Win32
//! behavior: the system does not deliver paint messages to invisible windows, `RDW_INTERNALPAINT`
//! only bypasses the *update region* requirement, not the *visibility* requirement. eframe runs
//! `App::update` / `raw_input_hook` only from `RedrawRequested`, which is produced by `WM_PAINT`,
//! so a `WM_CLOSE` posted to a hidden root `HWND` sits in the queue until the next show. To avoid
//! that deadlock we call `ShowWindow(SW_SHOWNA)` right before `PostMessageW(WM_CLOSE)` (brief
//! flash, no focus theft), which is the simplest way to guarantee a one-shot paint so eframe
//! drives the close path and `on_exit` runs as usual.

use std::collections::VecDeque;
use std::sync::Mutex;
#[cfg(windows)]
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::{AtomicBool, Ordering};

use titan_common::UiLang;
use titan_i18n::{Msg, t};

use crate::menu::{self, DesktopProduct};
use tray_icon::menu::{MenuEvent, MenuId};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

static PENDING_MENU_IDS: Mutex<VecDeque<MenuId>> = Mutex::new(VecDeque::new());
static TRAY_HANDLERS_REGISTERED: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
static WIN_TRAY_WAKE_HWND: AtomicIsize = AtomicIsize::new(0);

/// Set when the user chooses **Quit** from the tray menu; cleared in [`consume_windows_tray_quit_close_request`].
#[cfg(windows)]
static WIN_TRAY_QUIT_PENDING: AtomicBool = AtomicBool::new(false);

/// Root egui / winit window `HWND` (Win32). Required on **Windows** so tray/menu callbacks can wake
/// the event loop and drive show/hide while the UI is hidden. Call once from the `eframe` creation
/// closure (see `apps/titan-center/src/main.rs`, `apps/titan-host/src/main.rs`).
#[cfg(windows)]
pub fn set_windows_tray_wake_hwnd(hwnd: isize) {
    WIN_TRAY_WAKE_HWND.store(hwnd, Ordering::Release);
}

/// If tray **Quit** posted `WM_CLOSE`, the next `close_requested` must not be turned into “hide to tray”.
/// Call from `App::raw_input_hook` when `close_requested` is true **before** stripping
/// `ViewportEvent::Close`.
#[cfg(windows)]
pub fn consume_windows_tray_quit_close_request() -> bool {
    WIN_TRAY_QUIT_PENDING.swap(false, Ordering::AcqRel)
}

#[cfg(windows)]
fn stored_root_hwnd() -> Option<*mut std::ffi::c_void> {
    let v = WIN_TRAY_WAKE_HWND.load(Ordering::Acquire);
    if v == 0 {
        None
    } else {
        Some(v as *mut std::ffi::c_void)
    }
}

#[cfg(windows)]
fn post_wm_null_to_egui_root_hwnd() {
    let Some(hwnd) = stored_root_hwnd() else {
        return;
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;
    if unsafe { PostMessageW(hwnd, 0, 0, 0) } == 0 {
        tracing::warn!(
            "PostMessageW for tray wake failed: {}",
            std::io::Error::last_os_error()
        );
    }
}

/// Make the root window visible **without activation** so `WM_PAINT` can fire (MSDN: hidden
/// windows don't receive paint messages even with `RDW_INTERNALPAINT`); eframe's close path runs
/// from that single paint pass.
#[cfg(windows)]
fn win32_show_root_no_activate() {
    let Some(hwnd) = stored_root_hwnd() else {
        return;
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_SHOWNA, ShowWindow};
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNA);
    }
}

#[cfg(windows)]
fn post_wm_close_to_root() {
    let Some(hwnd) = stored_root_hwnd() else {
        return;
    };
    win32_show_root_no_activate();
    use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
    if unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) } == 0 {
        tracing::warn!(
            "PostMessageW(WM_CLOSE) for tray quit failed: {}",
            std::io::Error::last_os_error()
        );
    }
}

/// Directly restore + foreground the root window. Safe to call from tray / menu handlers (they run
/// on the UI thread via winit's message pump) and required on Windows because eframe's viewport
/// command queue is blocked while the window is hidden.
#[cfg(windows)]
fn win32_show_restore_foreground() {
    let Some(hwnd) = stored_root_hwnd() else {
        return;
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsIconic, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow,
    };
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        if IsIconic(hwnd) != 0 {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}

#[cfg(windows)]
fn win32_hide_root_window() {
    let Some(hwnd) = stored_root_hwnd() else {
        return;
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

fn push_pending_menu_id(id: MenuId) {
    let mut pending = match PENDING_MENU_IDS.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    pending.push_back(id);
}

#[cfg(target_os = "macos")]
fn schedule_egui_repaint_off_ui_thread(ctx: &egui::Context) {
    ctx.request_repaint();
}

#[cfg(not(target_os = "macos"))]
fn schedule_egui_repaint_off_ui_thread(ctx: &egui::Context) {
    #[cfg(windows)]
    post_wm_null_to_egui_root_hwnd();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        ctx.request_repaint();
    });
}

#[cfg(windows)]
fn handle_menu_event_show(show_id: &MenuId, ev: &MenuEvent) {
    if ev.id == *show_id {
        win32_show_restore_foreground();
    }
}

#[cfg(not(windows))]
fn handle_menu_event_show(_show_id: &MenuId, _ev: &MenuEvent) {}

#[cfg(windows)]
fn handle_menu_event_quit(quit_id: &MenuId, ev: &MenuEvent) {
    if ev.id != *quit_id {
        return;
    }
    WIN_TRAY_QUIT_PENDING.store(true, Ordering::Release);
    post_wm_close_to_root();
}

#[cfg(not(windows))]
fn handle_menu_event_quit(_quit_id: &MenuId, _ev: &MenuEvent) {}

fn register_menu_repaint_handler(ctx: &egui::Context, product: DesktopProduct) {
    let show_id = product.show_menu_id();
    let quit_id = product.quit_menu_id();
    let ctx_menu = ctx.clone();
    MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
        handle_menu_event_show(&show_id, &ev);
        handle_menu_event_quit(&quit_id, &ev);
        push_pending_menu_id(ev.id.clone());
        schedule_egui_repaint_off_ui_thread(&ctx_menu);
    }));
}

/// Left **single** click (all platforms) and **double** click open the main window.
fn tray_icon_requests_main_window(ev: &TrayIconEvent) -> bool {
    matches!(
        ev,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Down,
            ..
        } | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        }
    )
}

fn tray_icon_repaint_wakeup(ctx: &egui::Context, _ev: &TrayIconEvent) {
    #[cfg(not(target_os = "macos"))]
    if tray_icon_requests_main_window(_ev) {
        schedule_egui_repaint_off_ui_thread(ctx);
        return;
    }
    ctx.request_repaint();
}

#[cfg(target_os = "macos")]
fn apply_tray_icon_open_main_window(ctx: &egui::Context, show_id: &MenuId) {
    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
    push_pending_menu_id(show_id.clone());
}

#[cfg(windows)]
fn apply_tray_icon_open_main_window(_ctx: &egui::Context, show_id: &MenuId) {
    win32_show_restore_foreground();
    push_pending_menu_id(show_id.clone());
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn apply_tray_icon_open_main_window(_ctx: &egui::Context, show_id: &MenuId) {
    push_pending_menu_id(show_id.clone());
}

fn register_tray_icon_show_handler(ctx: &egui::Context, show_id: MenuId) {
    let ctx_tray = ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |ev| {
        if tray_icon_requests_main_window(&ev) {
            apply_tray_icon_open_main_window(&ctx_tray, &show_id);
        }
        tray_icon_repaint_wakeup(&ctx_tray, &ev);
    }));
}

fn register_tray_wakeup_impl(ctx: &egui::Context, product: DesktopProduct) {
    if TRAY_HANDLERS_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    let show_id = product.show_menu_id();
    register_menu_repaint_handler(ctx, product);
    register_tray_icon_show_handler(ctx, show_id);
}

/// Install global tray/menu hooks for **Titan Center** (same as [`register_tray_wakeup_impl`] with
/// Center ids).
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
            send_show_and_focus_root(ctx);
        } else if id == quit_id {
            *really_quitting = true;
            #[cfg(windows)]
            {
                WIN_TRAY_QUIT_PENDING.store(true, Ordering::Release);
                post_wm_close_to_root();
            }
            #[cfg(not(windows))]
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
            ctx.request_repaint();
        }
    }
    showed
}

/// Bring the root window back from tray: on Windows bypass the eframe viewport queue with Win32
/// `ShowWindow` / `SetForegroundWindow`; on other platforms use [`egui::ViewportCommand::Visible`]
/// + [`egui::ViewportCommand::Focus`].
fn send_show_and_focus_root(ctx: &egui::Context) {
    #[cfg(windows)]
    win32_show_restore_foreground();
    #[cfg(not(windows))]
    {
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
    }
    ctx.request_repaint();
}

/// Hide the main egui viewport into the tray. On **Windows** we call `ShowWindow(SW_HIDE)` on the
/// root `HWND` stored via [`set_windows_tray_wake_hwnd`], bypassing eframe's viewport command
/// queue (see egui #5229). On other platforms we send [`egui::ViewportCommand::Visible(false)`].
///
/// Caller handling a close event must first send [`egui::ViewportCommand::CancelClose`].
pub fn hide_main_window_to_tray(ctx: &egui::Context) {
    #[cfg(windows)]
    win32_hide_root_window();
    #[cfg(not(windows))]
    ctx.send_viewport_cmd_to(
        egui::ViewportId::ROOT,
        egui::ViewportCommand::Visible(false),
    );
    ctx.request_repaint_after_for(
        std::time::Duration::from_millis(250),
        egui::ViewportId::ROOT,
    );
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
