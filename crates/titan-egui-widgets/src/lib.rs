//! Shared egui UI primitives for **Titan Center** and **Titan Host** (buttons, cards, modals,
//! fields, inset dropdown). Palette lives in [`theme`]; keep values aligned with each app’s
//! layout constants when changing visuals.

mod buttons;
mod frames;
mod layout;
mod modal;
mod select_dropdown;
mod text_fields;

pub mod theme;

pub use buttons::{
    danger_preview_delete_button, preview_overlay_configure_button, primary_button_large,
    subtle_button, subtle_button_large, subtle_button_toolbar,
};
pub use frames::{form_field_row, opaque_dialog_frame, opaque_dialog_frame_ctx, section_card};
pub use layout::{preview_overlay_action_bar_rects, preview_overlay_action_bar_rects_three};
pub use modal::{OpaqueFrameSource, show_opaque_modal, show_settings_tool_window};
pub use select_dropdown::{InsetDropdownLayout, inset_single_select_dropdown};
pub use text_fields::{dialog_underline_text_row, dialog_underline_text_row_gap, multiline_inset};
