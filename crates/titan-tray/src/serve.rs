//! Tray for `titan-host serve`: no main window — only **Quit** stops the listener via a watch flag.

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
use std::time::Duration;

use tokio::sync::watch;
use tray_icon::menu::{MenuEvent, MenuId};
use tray_icon::TrayIconBuilder;

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
use tray_icon::TrayIconEvent;

use crate::menu::{self, DesktopProduct, MENU_HOST_QUIT};

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
fn poll_host_tray_until_quit(shutdown_tx: &watch::Sender<bool>, quit_id: &MenuId) {
    loop {
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == *quit_id {
                let _ = shutdown_tx.send(true);
                return;
            }
        }
        while TrayIconEvent::receiver().try_recv().is_ok() {}
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
fn windows_tray_host_thread(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let menu = match menu::build_tray_menu(DesktopProduct::Host) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("tray: menu build failed: {e}");
            return;
        }
    };
    let icon = crate::icon::tray_icon_for(DesktopProduct::Host);
    let _tray = match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(&tooltip)
        .with_icon(icon)
        .build()
    {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("system tray unavailable: {e}");
            return;
        }
    };
    let quit_id = MenuId::new(MENU_HOST_QUIT);
    poll_host_tray_until_quit(&shutdown_tx, &quit_id);
}

fn host_tooltip(tooltip: &str) -> String {
    if tooltip.is_empty() {
        "Titan".to_string()
    } else {
        tooltip.to_string()
    }
}

#[cfg(target_os = "linux")]
fn linux_build_host_tray(tooltip: &str) -> Result<tray_icon::TrayIcon, String> {
    let menu = menu::build_tray_menu(DesktopProduct::Host).map_err(|e| e.to_string())?;
    let icon = crate::icon::tray_icon_for(DesktopProduct::Host);
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .map_err(|e| e.to_string())
}

/// Windows (and other non-macOS desktops except Linux): tray thread polls [`MenuEvent`].
#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);
    std::thread::Builder::new()
        .name("titan-tray-host".to_string())
        .spawn(move || windows_tray_host_thread(shutdown_tx, tooltip))
        .expect("spawn titan-tray-host thread");
}

#[cfg(target_os = "macos")]
fn macos_set_accessory_activation_policy() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        app.finishLaunching();
        app.activate();
    }
}

#[cfg(target_os = "macos")]
fn macos_install_tray_icon_and_tick_runloop(tooltip: &str) -> Result<(), String> {
    let menu = menu::build_tray_menu(DesktopProduct::Host).map_err(|e| e.to_string())?;
    let icon = crate::icon::tray_icon_for(DesktopProduct::Host);
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(tooltip)
        .with_icon(icon)
        .with_icon_as_template(false)
        .build()
        .map_err(|e| e.to_string())?;
    std::mem::forget(tray);
    use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop};
    let _ = unsafe { CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, 0.05, true) };
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_install_host_tray_for_serve(
    shutdown_tx: &watch::Sender<bool>,
    tooltip: &str,
) -> Result<(), String> {
    macos_set_accessory_activation_policy();
    let quit_id = MenuId::new(MENU_HOST_QUIT);
    let shutdown_cb = shutdown_tx.clone();
    MenuEvent::set_event_handler(Some(move |ev: tray_icon::menu::MenuEvent| {
        if ev.id == quit_id {
            let _ = shutdown_cb.send(true);
        }
    }));
    macos_install_tray_icon_and_tick_runloop(tooltip)
}

/// macOS: `muda::Menu` / NSStatusItem must be created on the **main thread** — dispatch there, then
/// use [`MenuEvent::set_event_handler`] (no polling thread).
#[cfg(target_os = "macos")]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);
    let on_main = unsafe { libc::pthread_main_np() != 0 };
    let res = if on_main {
        macos_install_host_tray_for_serve(&shutdown_tx, &tooltip)
    } else {
        let stx = shutdown_tx.clone();
        let tip = tooltip.clone();
        dispatch::Queue::main().exec_sync(move || macos_install_host_tray_for_serve(&stx, &tip))
    };
    if let Err(e) = res {
        tracing::warn!("system tray unavailable: {e}");
    }
}

#[cfg(target_os = "linux")]
fn linux_gtk_host_tray_main(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    if gtk::init().is_err() {
        tracing::warn!("tray: gtk::init failed; system tray disabled on Linux");
        return;
    }
    let _tray = match linux_build_host_tray(&tooltip) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("system tray unavailable: {e}");
            return;
        }
    };
    let shutdown_cb = shutdown_tx.clone();
    let quit_id = MenuId::new(MENU_HOST_QUIT);
    MenuEvent::set_event_handler(Some(move |ev| {
        if ev.id == quit_id {
            let _ = shutdown_cb.send(true);
            gtk::main_quit();
        }
    }));
    gtk::main();
    MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
    let _ = _tray;
}

/// Linux: GTK thread + [`MenuEvent::set_event_handler`] so Quit does not contend with a second
/// [`MenuEvent::receiver`] (GTK menu runs on the GTK thread).
#[cfg(target_os = "linux")]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);
    std::thread::Builder::new()
        .name("titan-tray-host".to_string())
        .spawn(move || linux_gtk_host_tray_main(shutdown_tx, tooltip))
        .expect("spawn titan-tray-host thread");
}
