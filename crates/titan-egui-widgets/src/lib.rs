//! Shared egui UI primitives for **Titan Center** and **Titan Host** (buttons, cards, modals,
//! fields, inset dropdown). Palette lives in [`theme`]; keep values aligned with each app’s
//! layout constants when changing visuals.

mod buttons;
mod frames;
mod modal;
mod select_dropdown;
mod text_fields;

pub mod theme;

pub use buttons::{
    danger_preview_delete_button, preview_overlay_configure_button, primary_button_large,
    subtle_button, subtle_button_large, subtle_button_toolbar,
};
pub use frames::{form_field_row, opaque_dialog_frame, opaque_dialog_frame_ctx, section_card};
pub use modal::{show_opaque_modal, show_settings_tool_window, OpaqueFrameSource};
pub use select_dropdown::{inset_single_select_dropdown, InsetDropdownLayout};
pub use text_fields::{dialog_underline_text_row, dialog_underline_text_row_gap, multiline_inset};
