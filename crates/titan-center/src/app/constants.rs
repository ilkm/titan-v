//! Shared layout and palette constants for the center UI.

use egui::{CornerRadius, Shadow};

pub const PANEL_SPACING: f32 = 12.0;

/// Gap between the two main columns (Hosts/VMs, Monitor).
pub const CONTENT_COLUMN_GAP: f32 = 20.0;

/// Card surface — reads clearly against [`theme`] page background.
pub const CARD_SURFACE: egui::Color32 = egui::Color32::from_rgb(248, 250, 255);

/// Right-aligned form label column (fits short CJK labels).
pub const FORM_LABEL_WIDTH: f32 = 120.0;

/// YouTube-style host tile width (thumbnail uses full inner width, 16:9 height).
pub const HOST_TILE_WIDTH: f32 = 200.0;

/// Max width for main content (centered on wide windows).
pub const CONTENT_MAX_WIDTH: f32 = 960.0;

/// Fixed left nav width (fits up to ~6 CJK menu labels + padding).
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 158.0;

/// Min / max when nav width tracks the header title separator X.
pub const SIDEBAR_MIN_WIDTH: f32 = 120.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 300.0;

/// Card corner radius (modern, soft).
pub const CARD_CORNER_RADIUS: CornerRadius = CornerRadius::same(12);

/// Subtle elevation for cards (light theme).
#[must_use]
pub fn card_shadow() -> Shadow {
    Shadow {
        offset: [0, 2],
        blur: 14,
        spread: 0,
        color: egui::Color32::from_rgba_unmultiplied(15, 23, 42, 14),
    }
}
pub const PERSIST_KEY: &str = "titan_center_state_v1";
pub const VIRTUAL_SLOTS: usize = 8000;

/// Tech-blue accent for headings, links, and primary actions (light theme).
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
pub const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);

/// Left nav row hit target height (no frame fill; typography shows selection).
pub const NAV_ITEM_HEIGHT: f32 = 32.0;
pub const OK_GREEN: egui::Color32 = egui::Color32::from_rgb(22, 163, 74);
pub const ERR_ROSE: egui::Color32 = egui::Color32::from_rgb(220, 38, 38);

/// Danger-zone card (light background).
pub const DANGER_CARD_FILL: egui::Color32 = egui::Color32::from_rgb(255, 241, 242);
pub const DANGER_CARD_STROKE: egui::Color32 = egui::Color32::from_rgb(252, 165, 165);

/// Inline error / alert panel (light background).
pub const ALERT_PANEL_FILL: egui::Color32 = egui::Color32::from_rgb(254, 242, 242);
