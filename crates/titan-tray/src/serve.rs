//! Tray for `titan-host serve`: no main window — only **Quit** stops the listener via a watch flag.

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
use std::time::Duration;

use tokio::sync::watch;
use tray_icon::menu::{MenuEvent, MenuId};
use tray_icon::TrayIconBuilder;

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
use tray_icon::TrayIconEvent;

use crate::menu::{self, DesktopProduct, MENU_HOST_QUIT};

fn host_tooltip(tooltip: &str) -> String {
    if tooltip.is_empty() {
        "Titan".to_string()
    } else {
        tooltip.to_string()
    }
}

/// Windows (and other non-macOS desktops except Linux): tray thread polls [`MenuEvent`].
#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);
    std::thread::Builder::new()
        .name("titan-tray-host".to_string())
        .spawn(move || {
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
            loop {
                while let Ok(ev) = MenuEvent::receiver().try_recv() {
                    if ev.id == quit_id {
                        let _ = shutdown_tx.send(true);
                        return;
                    }
                }
                while TrayIconEvent::receiver().try_recv().is_ok() {}
                std::thread::sleep(Duration::from_millis(200));
            }
        })
        .expect("spawn titan-tray-host thread");
}

/// macOS: `muda::Menu` / NSStatusItem must be created on the **main thread** — dispatch there, then
/// use [`MenuEvent::set_event_handler`] (no polling thread).
#[cfg(target_os = "macos")]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);

    fn install(shutdown_tx: &watch::Sender<bool>, tooltip: &str) -> Result<(), String> {
        // Headless `titan-host` has no NSWindow: accessory policy + normal launch hooks so the
        // status item can appear in the menu bar.
        {
            use objc2::MainThreadMarker;
            use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
            if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                app.finishLaunching();
                app.activate();
            }
        }

        let quit_id = MenuId::new(MENU_HOST_QUIT);
        let shutdown_cb = shutdown_tx.clone();
        MenuEvent::set_event_handler(Some(move |ev: tray_icon::menu::MenuEvent| {
            if ev.id == quit_id {
                let _ = shutdown_cb.send(true);
            }
        }));

        let menu = menu::build_tray_menu(DesktopProduct::Host).map_err(|e| e.to_string())?;
        let icon = crate::icon::tray_icon_for(DesktopProduct::Host);
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(tooltip)
            .with_icon(icon)
            // Solid-color RGBA reads poorly as a template image on the menu bar; use full color.
            .with_icon_as_template(false)
            .build()
            .map_err(|e| e.to_string())?;
        std::mem::forget(tray);

        // Pump the main CFRunLoop briefly so AppKit can commit menu-bar layout under Tokio.
        use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop};
        let _ = unsafe { CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, 0.05, true) };

        Ok(())
    }

    let on_main = unsafe { libc::pthread_main_np() != 0 };
    let res = if on_main {
        install(&shutdown_tx, &tooltip)
    } else {
        let stx = shutdown_tx.clone();
        let tip = tooltip.clone();
        dispatch::Queue::main().exec_sync(move || install(&stx, &tip))
    };

    if let Err(e) = res {
        tracing::warn!("system tray unavailable: {e}");
    }
}

/// Linux: GTK thread + [`MenuEvent::set_event_handler`] so Quit does not contend with a second
/// [`MenuEvent::receiver`] (GTK menu runs on the GTK thread).
#[cfg(target_os = "linux")]
pub fn spawn_tray_shutdown_for_serve(shutdown_tx: watch::Sender<bool>, tooltip: String) {
    let tooltip = host_tooltip(&tooltip);
    std::thread::Builder::new()
        .name("titan-tray-host".to_string())
        .spawn(move || {
            if gtk::init().is_err() {
                tracing::warn!("tray: gtk::init failed; system tray disabled on Linux");
                return;
            }
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
        })
        .expect("spawn titan-tray-host thread");
}
