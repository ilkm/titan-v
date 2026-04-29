//! Layout tokens aligned with Titan Center (`apps/titan-center/src/app/constants.rs`).

use egui::Color32;

pub use titan_egui_widgets::theme::{ACCENT, card_shadow};

/// Soft cap for main content column (same as center).
pub const CONTENT_MAX_WIDTH: f32 = 960.0;

/// Fixed left nav width (matches center).
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 158.0;

/// Left nav row hit target height.
pub const NAV_ITEM_HEIGHT: f32 = 32.0;

pub const ERR_ROSE: Color32 = Color32::from_rgb(220, 38, 38);
