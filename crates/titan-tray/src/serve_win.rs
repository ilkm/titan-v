//! Win32 message pump for the dedicated `serve` tray thread.
//!
//! tray-icon registers a hidden `HWND` on this thread; shell notify callbacks require
//! [`PeekMessageW`] / [`DispatchMessageW`] on that same thread (see tray-icon crate docs).

use std::mem::MaybeUninit;
use std::time::Duration;

use tokio::sync::watch;
use tray_icon::TrayIconEvent;
use tray_icon::menu::{MenuEvent, MenuId};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};

pub(crate) fn poll_host_tray_until_quit(shutdown_tx: &watch::Sender<bool>, quit_id: &MenuId) {
    loop {
        pump_thread_messages();
        if quit_seen(shutdown_tx, quit_id) {
            return;
        }
        drain_tray_icon_events();
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn pump_thread_messages() {
    unsafe {
        let hwnd: HWND = std::ptr::null_mut();
        loop {
            let mut msg = MaybeUninit::<MSG>::uninit();
            if PeekMessageW(msg.as_mut_ptr(), hwnd, 0, 0, PM_REMOVE) == 0 {
                break;
            }
            let msg = msg.assume_init();
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn quit_seen(shutdown_tx: &watch::Sender<bool>, quit_id: &MenuId) -> bool {
    while let Ok(ev) = MenuEvent::receiver().try_recv() {
        if ev.id == *quit_id {
            let _ = shutdown_tx.send(true);
            return true;
        }
    }
    false
}

fn drain_tray_icon_events() {
    while TrayIconEvent::receiver().try_recv().is_ok() {}
}
