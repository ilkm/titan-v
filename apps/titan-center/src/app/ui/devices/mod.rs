//! Standalone device-management cards (Connect tab); host grid was removed from Settings.

mod add_host_dialog;
mod device_card;
mod helpers;
mod host_config_window;
mod tab;
mod tofu_dialog;

pub(crate) use helpers::{device_mgmt_card_height_hint, device_mgmt_cols_and_card_width};
