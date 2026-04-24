//! Shared layout and palette constants for the center UI.

use egui::{CornerRadius, Shadow};

pub const PANEL_SPACING: f32 = 12.0;

/// Gap between the two main columns (Hosts/VMs, Monitor).
pub const CONTENT_COLUMN_GAP: f32 = 20.0;

/// Card surface — reads clearly against [`theme`] page background.
pub const CARD_SURFACE: egui::Color32 = egui::Color32::from_rgb(248, 250, 255);

/// Max width of the label column in [`widgets::form_field_row`] (CJK lines wrap; left-aligned).
pub const FORM_LABEL_WIDTH: f32 = 132.0;

/// Minimum host tile width in the settings grid; actual width scales with window.
pub const HOST_TILE_MIN_WIDTH: f32 = 168.0;

/// Device management grid: horizontal gap between cards.
pub const DEVICE_CARD_GAP: f32 = 12.0;
/// Device management: one card width lower bound (drives max columns ≤ 6).
pub const DEVICE_CARD_MIN_WIDTH: f32 = 300.0;
/// Device management: one card width upper bound (drives min columns ≥ 1).
pub const DEVICE_CARD_MAX_WIDTH: f32 = 480.0;

/// How often the center requests a fresh desktop JPEG from each known host (Connect tab).
pub const DESKTOP_PREVIEW_POLL_SECS: f32 = 2.0;

/// If no telemetry push arrives for this long while the session is marked live, clear the live
/// telemetry flag (VM inventory may be stale). **Device card online** is driven by periodic
/// Hello reachability, not by this timer. Pushes include periodic `HostResourceLive` plus pushes
/// after control-plane responses (`HostTelemetry`).
pub const TELEMETRY_STALE_AFTER_SECS: f64 = 120.0;

/// Max concurrent telemetry TCP readers (one per distinct `host_key`).
pub const TELEMETRY_MAX_CONCURRENT: usize = 8;

/// Background Hello probe interval for every saved device (updates per-card online when not on telemetry).
pub const REACHABILITY_PROBE_SECS: f32 = 3.0;

/// Manual add-host: Hello over control TCP (Tokio `timeout`); use a **multi-thread** runtime so the timer fires reliably.
pub const ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS: u64 = 5;
/// UI watchdog if the worker never posts [`NetUiMsg::AddHostVerifyDone`] (must be ≥ hello timeout + slack).
pub const ADD_HOST_VERIFY_UI_DEADLINE_SECS: u64 = ADD_HOST_VERIFY_HELLO_TIMEOUT_SECS + 2;

/// Soft cap for main content column; effective width grows with window (see `effective_content_width`).
pub const CONTENT_MAX_WIDTH: f32 = 960.0;

/// Fixed left nav width (longest CJK labels + padding). Intentionally not tied to the header title
/// so switching UI language does not resize the nav or shift the main window layout.
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 158.0;

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
