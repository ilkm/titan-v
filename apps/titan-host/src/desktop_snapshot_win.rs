//! Primary-display capture on Windows using GDI with `SRCCOPY | CAPTUREBLT`.
//!
//! `CAPTUREBLT` includes layered windows in the blit so the pixmap matches the visually top
//! composited desktop (MSDN `StretchBlt` raster operations). Callers must satisfy the same GDI
//! constraints as the `screenshots` crate. Win32 calls are `unsafe` and assume the OS returns
//! valid handles until explicitly released in `Drop` or after successful reads.

#![allow(unsafe_code)]

use std::mem;

use image::RgbaImage;
use screenshots::display_info::DisplayInfo;
use windows::Win32::Graphics::Gdi::{
    BITMAPINFO, BITMAPINFOHEADER, CAPTUREBLT, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateDCW, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, GetMonitorInfoW, HBITMAP, HDC,
    HGDIOBJ, HMONITOR, MONITORINFO, MONITORINFOEXW, RGBQUAD, ROP_CODE, SRCCOPY, STRETCH_HALFTONE,
    SelectObject, SetStretchBltMode, StretchBlt,
};
use windows::core::PCWSTR;

struct DcGuard(HDC);

impl DcGuard {
    fn new_screen(device: *const u16) -> Result<Self, String> {
        // SAFETY: `device` points to a null-terminated monitor device name from Win32 monitor info.
        let hdc = unsafe { CreateDCW(PCWSTR(device), PCWSTR(device), PCWSTR::null(), None) };
        if hdc.is_invalid() {
            return Err("CreateDCW failed".to_string());
        }
        Ok(Self(hdc))
    }

    fn new_compatible(src: HDC) -> Result<Self, String> {
        // SAFETY: `src` is a valid display DC owned by `DcGuard`.
        let hdc = unsafe { CreateCompatibleDC(src) };
        if hdc.is_invalid() {
            return Err("CreateCompatibleDC failed".to_string());
        }
        Ok(Self(hdc))
    }

    fn raw(&self) -> HDC {
        self.0
    }
}

impl Drop for DcGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: `self.0` is a DC created by this guard and not used after drop.
            unsafe {
                let _ = DeleteDC(self.0);
            }
        }
    }
}

struct BitmapGuard(HBITMAP);

impl BitmapGuard {
    fn new(dc: HDC, w: i32, h: i32) -> Result<Self, String> {
        // SAFETY: `dc` is a valid screen DC and dimensions are sourced from monitor bounds.
        let hb = unsafe { CreateCompatibleBitmap(dc, w, h) };
        if hb.is_invalid() {
            return Err("CreateCompatibleBitmap failed".to_string());
        }
        Ok(Self(hb))
    }

    fn raw(&self) -> HBITMAP {
        self.0
    }
}

impl Drop for BitmapGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: bitmap handle was created by `CreateCompatibleBitmap` and is uniquely owned here.
            unsafe {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

struct SelectedBitmap {
    mem: HDC,
    previous: HGDIOBJ,
}

impl SelectedBitmap {
    unsafe fn select(mem: HDC, bmp: HBITMAP) -> Result<Self, String> {
        // SAFETY: `mem` is a memory DC and `bmp` remains alive for this guard's lifetime.
        let previous = unsafe { SelectObject(mem, bmp) };
        if previous.is_invalid() {
            return Err("SelectObject failed".to_string());
        }
        Ok(Self { mem, previous })
    }
}

impl Drop for SelectedBitmap {
    fn drop(&mut self) {
        // SAFETY: restoring the previous selected object back into the same memory DC.
        unsafe {
            let _ = SelectObject(self.mem, self.previous);
        }
    }
}

pub fn capture_primary_display_rgba() -> Result<RgbaImage, String> {
    let d = primary_display()?;
    let (w, h) = scaled_dims(&d);
    capture_via_gdi(d.raw_handle, w, h)
}

fn primary_display() -> Result<DisplayInfo, String> {
    let v = DisplayInfo::all().map_err(|e| e.to_string())?;
    v.iter()
        .find(|x| x.is_primary)
        .or_else(|| v.first())
        .copied()
        .ok_or_else(|| "no displays found".to_string())
}

fn scaled_dims(d: &DisplayInfo) -> (i32, i32) {
    let w = ((d.width as f32) * d.scale_factor) as i32;
    let h = ((d.height as f32) * d.scale_factor) as i32;
    (w, h)
}

fn monitor_device_utf16(hm: HMONITOR) -> Result<[u16; 32], String> {
    // SAFETY: zeroed `MONITORINFOEXW` is immediately initialized via `cbSize` before Win32 call.
    let mut info: MONITORINFOEXW = unsafe { mem::zeroed() };
    info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    let p = (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>();
    // SAFETY: `p` points to writable `MONITORINFOEXW` storage with correct `cbSize`.
    unsafe {
        GetMonitorInfoW(hm, p)
            .ok()
            .map_err(|e| format!("GetMonitorInfoW: {e}"))?;
    }
    Ok(info.szDevice)
}

fn bitmap_info(w: i32, h: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default(); 1],
    }
}

fn rows_bottom_up_to_rgba(w: u32, h: u32, data: Vec<u8>) -> Result<RgbaImage, String> {
    let row = w as usize * 4;
    let mut rows: Vec<Vec<u8>> = data.chunks_exact(row).map(|c| c.to_vec()).collect();
    rows.reverse();
    let mut flat: Vec<u8> = rows.into_iter().flatten().collect();
    for px in flat.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    RgbaImage::from_raw(w, h, flat).ok_or_else(|| "invalid capture buffer".to_string())
}

fn capture_via_gdi(hm: HMONITOR, w: i32, h: i32) -> Result<RgbaImage, String> {
    let sz = monitor_device_utf16(hm)?;
    let dev = sz.as_ptr();
    let screen = DcGuard::new_screen(dev)?;
    let mem = DcGuard::new_compatible(screen.raw())?;
    let bmp = BitmapGuard::new(screen.raw(), w, h)?;
    // SAFETY: memory DC + bitmap selection are valid; guard ensures previous object is restored.
    let _sel = unsafe { SelectedBitmap::select(mem.raw(), bmp.raw())? };
    // SAFETY: source and destination DCs are valid and dimensions match allocated bitmap size.
    unsafe {
        let _ = SetStretchBltMode(screen.raw(), STRETCH_HALFTONE);
        StretchBlt(
            mem.raw(),
            0,
            0,
            w,
            h,
            screen.raw(),
            0,
            0,
            w,
            h,
            ROP_CODE(SRCCOPY.0 | CAPTUREBLT.0),
        )
        .ok()
        .map_err(|e| format!("StretchBlt: {e}"))?;
    }
    let pixels = read_dibits(mem.raw(), bmp.raw(), w, h)?;
    rows_bottom_up_to_rgba(w as u32, h as u32, pixels)
}

fn read_dibits(mem: HDC, bmp: HBITMAP, w: i32, h: i32) -> Result<Vec<u8>, String> {
    let mut bmi = bitmap_info(w, h);
    let len = (w as i64).saturating_mul(h as i64).saturating_mul(4) as usize;
    let mut data = vec![0u8; len];
    let buf = data.as_mut_ptr().cast();
    // SAFETY: `buf` points to writable `len` bytes and `bmi` describes a 32-bit RGB destination.
    let lines = unsafe { GetDIBits(mem, bmp, 0, h as u32, Some(buf), &mut bmi, DIB_RGB_COLORS) };
    if lines == 0 {
        return Err("GetDIBits failed".to_string());
    }
    Ok(data)
}
