//! Light-theme palette and layout tokens used by shared widgets.
//!
//! Values align with each app’s `constants` where Center and Host should look the same; apps keep
//! their own constants for non-widget layout.

use egui::{Color32, CornerRadius, Shadow};

/// Vertical gap after section card bodies (see `section_card` in this crate).
pub const PANEL_SPACING: f32 = 12.0;

/// Card surface — reads clearly against typical page backgrounds.
pub const CARD_SURFACE: Color32 = Color32::from_rgb(248, 250, 255);

/// Max width of the label column in `form_field_row` (this crate).
pub const FORM_LABEL_WIDTH: f32 = 132.0;

/// Card corner radius (modern, soft).
pub const CARD_CORNER_RADIUS: CornerRadius = CornerRadius::same(12);

/// Subtle elevation for cards (light theme).
#[must_use]
pub fn card_shadow() -> Shadow {
    Shadow {
        offset: [0, 2],
        blur: 14,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(15, 23, 42, 14),
    }
}

/// Tech-blue accent for headings, links, and primary actions (light theme).
pub const ACCENT: Color32 = Color32::from_rgb(37, 99, 235);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(29, 78, 216);

/// Default value text on light inset fields and opaque dialogs (slate-900).
pub const FORM_VALUE_TEXT: Color32 = Color32::from_rgb(15, 23, 42);
