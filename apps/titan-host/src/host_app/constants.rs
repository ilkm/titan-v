//! Layout tokens aligned with Titan Center (`apps/titan-center/src/app/constants.rs`).

use egui::Color32;

pub use titan_egui_widgets::theme::{ACCENT, CARD_CORNER_RADIUS, CARD_SURFACE, card_shadow};

/// Device-style window cards: horizontal gap (matches center Connect tab).
pub const DEVICE_CARD_GAP: f32 = 12.0;
/// Lower bound for one card width (drives max columns).
pub const DEVICE_CARD_MIN_WIDTH: f32 = 300.0;
/// Upper bound for one card width (drives min columns).
pub const DEVICE_CARD_MAX_WIDTH: f32 = 480.0;

/// Soft cap for main content column (same as center).
pub const CONTENT_MAX_WIDTH: f32 = 960.0;

/// Fixed left nav width (matches center).
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 158.0;

/// Left nav row hit target height.
pub const NAV_ITEM_HEIGHT: f32 = 32.0;

pub const ERR_ROSE: Color32 = Color32::from_rgb(220, 38, 38);
