//! Rounded-rectangle hit test for tray RGBA (axis-aligned, uniform corner radius).

pub(crate) fn inside_round_rect(x: i32, y: i32, w: i32, h: i32, rad: i32) -> bool {
    if x < 0 || y < 0 || x >= w || y >= h {
        return false;
    }
    let r = rad.min(w / 2).min(h / 2).max(0);
    if r == 0 {
        return true;
    }
    if x < r && y < r {
        return corner_disk(x, y, r, r, r);
    }
    if x >= w - r && y < r {
        return corner_disk(x, y, w - r, r, r);
    }
    if x < r && y >= h - r {
        return corner_disk(x, y, r, h - r, r);
    }
    if x >= w - r && y >= h - r {
        return corner_disk(x, y, w - r, h - r, r);
    }
    true
}

fn corner_disk(x: i32, y: i32, cx: i32, cy: i32, r: i32) -> bool {
    let dx = (x - cx) as f32;
    let dy = (y - cy) as f32;
    dx.hypot(dy) <= r as f32 + 1.0e-3
}
