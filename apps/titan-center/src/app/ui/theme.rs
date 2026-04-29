//! egui light theme (tech blue) for Titan Center.

use std::borrow::Cow;

use egui::{FontData, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2};
use fontdb::{Database, Family, Query};

use crate::app::constants::{ACCENT, ACCENT_DIM, CARD_SURFACE, ERR_ROSE};

const CJK_FONT_ID: &str = "titan_cjk_system";

fn cjk_font_family_names() -> &'static [&'static str] {
    &[
        "PingFang SC",
        "PingFang C",
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
    ]
}

fn query_named_font_bytes(db: &Database, name: &str) -> Option<(Vec<u8>, u32)> {
    let query = Query {
        families: &[Family::Name(name)],
        ..Default::default()
    };
    let id = db.query(&query)?;
    db.with_face_data(id, |data, face_index| (data.to_vec(), face_index))
        .filter(|(bytes, _)| !bytes.is_empty())
}

/// Prefer common CJK-capable system fonts so Chinese (and mixed) UI text renders instead of tofu.
fn system_cjk_font() -> Option<(Vec<u8>, u32)> {
    let mut db = Database::new();
    db.load_system_fonts();
    for &name in cjk_font_family_names() {
        if let Some(out) = query_named_font_bytes(&db, name) {
            return Some(out);
        }
    }
    None
}

fn install_cjk_font_fallback(ctx: &egui::Context) {
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

fn apply_light_visuals(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.dark_mode = false;
    // Page canvas: cooler gray so cards / nav read as layered surfaces.
    let page = egui::Color32::from_rgb(232, 236, 244);
    v.window_fill = page;
    v.panel_fill = page;
    v.extreme_bg_color = egui::Color32::from_rgb(220, 228, 242);
    v.faint_bg_color = CARD_SURFACE;
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
    // Side nav vs. central separator and other noninteractive outlines: neutral light gray.
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, egui::Color32::from_rgb(218, 220, 224));
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(241, 245, 249);
    v.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(203, 213, 225);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(224, 242, 254);
    v.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(147, 197, 253);
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(219, 234, 254);
    v.selection.bg_fill = egui::Color32::from_rgb(191, 219, 254);
    v.selection.stroke = Stroke::new(1.0, ACCENT_DIM);
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = egui::Color32::from_rgb(180, 120, 20);
    v.error_fg_color = ERR_ROSE;
    ctx.set_visuals(v);
}

fn apply_text_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 10.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.menu_margin = Margin::same(10);
    style.spacing.scroll = egui::style::ScrollStyle {
        floating: false,
        ..style.spacing.scroll
    };
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}

pub fn apply_center_theme(ctx: &egui::Context) {
    install_cjk_font_fallback(ctx);
    apply_light_visuals(ctx);
    apply_text_styles(ctx);
}
