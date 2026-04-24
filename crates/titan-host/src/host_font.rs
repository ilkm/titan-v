//! System CJK font fallback for egui (same strategy as `titan-center`).

use std::borrow::Cow;

use egui::{FontData, FontFamily};
use fontdb::{Database, Family, Query};

const CJK_FONT_ID: &str = "titan_host_cjk_system";

fn system_cjk_font() -> Option<(Vec<u8>, u32)> {
    let mut db = Database::new();
    db.load_system_fonts();

    const NAMES: &[&str] = &[
        "PingFang SC",
        "PingFang TC",
        "Heiti SC",
        "Songti SC",
        "STHeiti",
        "Microsoft YaHei",
        "Microsoft YaHei UI",
        "SimHei",
        "Noto Sans CJK SC",
        "Noto Sans SC",
        "Source Han Sans SC",
        "WenQuanYi Micro Hei",
        "Noto Sans CJK JP",
    ];

    for &name in NAMES {
        let query = Query {
            families: &[Family::Name(name)],
            ..Default::default()
        };
        let Some(id) = db.query(&query) else {
            continue;
        };
        if let Some((bytes, face_index)) =
            db.with_face_data(id, |data, face_index| (data.to_vec(), face_index))
        {
            if !bytes.is_empty() {
                return Some((bytes, face_index));
            }
        }
    }

    None
}

/// Install a CJK-capable system font into [`egui::FontFamily::Proportional`] / `Monospace` stacks.
pub fn install_cjk_fonts(ctx: &egui::Context) {
    let Some((bytes, face_index)) = system_cjk_font() else {
        tracing::warn!(
            "no CJK system font found; install a Chinese-capable font (e.g. Noto Sans CJK SC)"
        );
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        CJK_FONT_ID.to_owned(),
        FontData {
            font: Cow::Owned(bytes),
            index: face_index,
            tweak: Default::default(),
        }
        .into(),
    );

    if let Some(stack) = fonts.families.get_mut(&FontFamily::Proportional) {
        stack.push(CJK_FONT_ID.to_owned());
    }
    if let Some(stack) = fonts.families.get_mut(&FontFamily::Monospace) {
        stack.push(CJK_FONT_ID.to_owned());
    }

    ctx.set_fonts(fonts);
}
