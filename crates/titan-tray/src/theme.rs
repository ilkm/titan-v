//! OS tray / menu bar color theme detection.
//!
//! We ask the shell whether the tray bar background is currently **Dark** or **Light**, so the
//! tray icon's chip color blends with the bar while the letter stays maximally contrasting. The
//! query is cheap (a single registry read on Windows, one `objc_msgSend` chain on macOS); call
//! [`current_tray_theme`] on demand (e.g. once per UI frame) and rebuild the tray icon when the
//! result changes.
//!
//! Sources:
//! - **Windows**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\SystemUsesLightTheme`
//!   (`REG_DWORD`: `0` = dark, `1` = light). This is the value Explorer reads to theme the
//!   taskbar and notification area; `AppsUseLightTheme` is a separate per-app hint we do not use.
//! - **macOS**: `NSApp.effectiveAppearance.name`. Dark appearances include `Dark` in their name
//!   (e.g. `NSAppearanceNameDarkAqua`, `NSAppearanceNameVibrantDark`), matching the classic
//!   `defaults read -g AppleInterfaceStyle == "Dark"` check with immediate accent tracking.
//! - Other platforms default to [`TrayTheme::Dark`] which matches the common panel style on Linux.

/// OS tray / menu bar color theme.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum TrayTheme {
    /// Light bar (white-ish background) — chip is light, glyph is dark.
    Light,
    /// Dark bar (near-black background) — chip is dark, glyph is light.
    #[default]
    Dark,
}

impl TrayTheme {
    /// Tray "chip" body RGBA — same color family as the bar background so the shape *blends*.
    pub(crate) fn chip_color(self) -> [u8; 4] {
        match self {
            Self::Dark => [0, 0, 0, 255],
            Self::Light => [255, 255, 255, 255],
        }
    }

    /// Glyph RGB — maximum contrast against [`Self::chip_color`] so the letter is readable.
    pub(crate) fn glyph_color(self) -> [u8; 3] {
        match self {
            Self::Dark => [255, 255, 255],
            Self::Light => [0, 0, 0],
        }
    }
}

/// Query the OS for the current tray color theme. Falls back to [`TrayTheme::Dark`] on error.
#[cfg(windows)]
pub fn current_tray_theme() -> TrayTheme {
    windows_tray_theme()
}

/// Query the OS for the current tray color theme. Falls back to [`TrayTheme::Dark`] on error.
#[cfg(target_os = "macos")]
pub fn current_tray_theme() -> TrayTheme {
    macos_tray_theme()
}

/// Fallback when neither Windows nor macOS; matches most Linux panel themes.
#[cfg(not(any(windows, target_os = "macos")))]
pub fn current_tray_theme() -> TrayTheme {
    TrayTheme::Dark
}

#[cfg(windows)]
fn windows_tray_theme() -> TrayTheme {
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, RegCloseKey, RegOpenKeyExW,
    };
    let subkey = utf16_z(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let value = utf16_z("SystemUsesLightTheme");
    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return TrayTheme::Dark;
        }
        let result = read_reg_dword(hkey, value.as_ptr())
            .map(|v| {
                if v == 1 {
                    TrayTheme::Light
                } else {
                    TrayTheme::Dark
                }
            })
            .unwrap_or(TrayTheme::Dark);
        let _ = RegCloseKey(hkey);
        result
    }
}

#[cfg(windows)]
unsafe fn read_reg_dword(
    hkey: windows_sys::Win32::System::Registry::HKEY,
    value_name: *const u16,
) -> Option<u32> {
    use windows_sys::Win32::System::Registry::RegQueryValueExW;
    let mut data: u32 = 0;
    let mut size: u32 = 4;
    let rc = unsafe {
        RegQueryValueExW(
            hkey,
            value_name,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            (&mut data as *mut u32) as *mut u8,
            &mut size,
        )
    };
    (rc == 0).then_some(data)
}

#[cfg(windows)]
fn utf16_z(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "macos")]
fn macos_tray_theme() -> TrayTheme {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    unsafe {
        let Some(cls) = class_by_name("NSApplication") else {
            return TrayTheme::Dark;
        };
        let app: *mut AnyObject = msg_send![cls, sharedApplication];
        if app.is_null() {
            return TrayTheme::Dark;
        }
        let appearance: *mut AnyObject = msg_send![app, effectiveAppearance];
        if appearance.is_null() {
            return TrayTheme::Dark;
        }
        appearance_theme(appearance)
    }
}

/// Decide [`TrayTheme`] from a non-null `NSAppearance*`. Dark names contain the `Dark` substring.
#[cfg(target_os = "macos")]
unsafe fn appearance_theme(appearance: *mut objc2::runtime::AnyObject) -> TrayTheme {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    let name: *mut AnyObject = unsafe { msg_send![appearance, name] };
    if name.is_null() {
        return TrayTheme::Light;
    }
    let cstr: *const i8 = unsafe { msg_send![name, UTF8String] };
    if cstr.is_null() {
        return TrayTheme::Light;
    }
    let bytes = unsafe { std::ffi::CStr::from_ptr(cstr).to_bytes() };
    if bytes.windows(4).any(|w| w == b"Dark") {
        TrayTheme::Dark
    } else {
        TrayTheme::Light
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn class_by_name(name: &str) -> Option<&'static objc2::runtime::AnyClass> {
    let c = std::ffi::CString::new(name).ok()?;
    objc2::runtime::AnyClass::get(c.as_c_str())
}
