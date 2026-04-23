//! Center UI: persisted host table, control-plane (multi-message TCP), VM inventory, scaled grids.

mod constants;
mod discovery;
mod i18n;
mod net_client;
mod net_msg;
mod panels_control;
mod panels_danger;
mod panels_hosts;
mod panels_inventory;
mod panels_misc;
mod panels_spoof;
mod persist_data;
mod spawn;
mod theme;
mod widgets;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use egui::{
    Align, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextStyle, TextWrapMode,
    WidgetText,
};

pub use persist_data::{CenterPersist, HostEndpoint, NavTab};

use self::constants::{
    card_shadow, ACCENT, CONTENT_COLUMN_GAP, CONTENT_MAX_WIDTH, NAV_ITEM_HEIGHT, PANEL_SPACING,
    PERSIST_KEY, SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH,
};
use self::i18n::{Msg, UiLang};
use self::net_msg::NetUiMsg;
use self::persist_data::{
    default_discovery_interval_secs, default_discovery_udp_port, default_list_vms_poll_secs,
};
use self::theme::apply_center_theme;

/// Center manager application state (UI thread).
pub struct CenterApp {
    pub(crate) ctx: egui::Context,
    /// Screen-space X of the vertical rule after the brand title (drives nav width alignment).
    pub(crate) header_sep_center_x: f32,
    pub(crate) endpoints: Vec<HostEndpoint>,
    pub(crate) selected_host: usize,
    pub(crate) accounts: Vec<String>,
    pub(crate) proxy_labels: Vec<String>,
    pub(crate) vm_inventory: Vec<titan_common::VmBrief>,
    pub(crate) bulk_vm_names: String,
    pub(crate) pending_confirm_stop: bool,
    pub(crate) pending_confirm_start: bool,
    pub(crate) last_action: String,
    pub(crate) control_addr: String,
    pub(crate) net_tx: Sender<NetUiMsg>,
    pub(crate) net_rx: Receiver<NetUiMsg>,
    pub(crate) net_busy: bool,
    pub(crate) host_connected: bool,
    pub(crate) last_capabilities: String,
    pub(crate) last_net_error: String,
    pub(crate) last_script_version: String,
    pub(crate) list_vms_auto_refresh: bool,
    pub(crate) list_vms_poll_secs: u32,
    pub(crate) list_vms_poll_accum: f32,
    pub(crate) discovery_gen: Arc<AtomicU64>,
    pub(crate) discovery_prev_broadcast: bool,
    pub(crate) discovery_broadcast: bool,
    pub(crate) discovery_interval_secs: u32,
    pub(crate) discovery_udp_port: u16,
    pub(crate) spoof_target_vm: String,
    pub(crate) spoof_dynamic_mac: bool,
    pub(crate) spoof_disable_checkpoints: bool,
    pub(crate) pending_spoof_confirm_apply: bool,
    pub(crate) agent_register_vm: String,
    pub(crate) agent_register_addr: String,
    pub(crate) ui_lang: UiLang,
    pub(crate) settings_open: bool,
    pub(crate) active_nav: NavTab,
}

impl CenterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_center_theme(&cc.egui_ctx);
        let (net_tx, net_rx) = mpsc::channel();
        let persist: CenterPersist = cc
            .storage
            .and_then(|s| s.get_string(PERSIST_KEY))
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_else(|| CenterPersist {
                endpoints: vec![HostEndpoint {
                    label: "local".into(),
                    addr: "127.0.0.1:7788".into(),
                    last_caps: String::new(),
                    last_vm_count: 0,
                }],
                accounts: vec!["demo-account-1".into()],
                proxy_labels: vec!["proxy-pool-a".into()],
                last_script_version: String::new(),
                list_vms_auto_refresh: false,
                list_vms_poll_secs: default_list_vms_poll_secs(),
                discovery_broadcast: false,
                discovery_interval_secs: default_discovery_interval_secs(),
                discovery_udp_port: default_discovery_udp_port(),
                ui_lang: UiLang::default(),
                active_nav: NavTab::default(),
            });
        let control_addr = persist
            .endpoints
            .first()
            .map(|e| e.addr.clone())
            .unwrap_or_else(|| "127.0.0.1:7788".into());
        let ui_lang = persist.ui_lang;
        let active_nav = persist.active_nav;
        Self {
            ctx: cc.egui_ctx.clone(),
            header_sep_center_x: 0.0,
            endpoints: persist.endpoints,
            selected_host: 0,
            accounts: persist.accounts,
            proxy_labels: persist.proxy_labels,
            last_script_version: persist.last_script_version,
            list_vms_auto_refresh: persist.list_vms_auto_refresh,
            list_vms_poll_secs: persist.list_vms_poll_secs.max(5),
            list_vms_poll_accum: 0.0,
            discovery_gen: Arc::new(AtomicU64::new(0)),
            discovery_prev_broadcast: false,
            discovery_broadcast: persist.discovery_broadcast,
            discovery_interval_secs: persist.discovery_interval_secs.max(1),
            discovery_udp_port: persist.discovery_udp_port,
            spoof_target_vm: String::new(),
            spoof_dynamic_mac: true,
            spoof_disable_checkpoints: false,
            pending_spoof_confirm_apply: false,
            agent_register_vm: String::new(),
            agent_register_addr: String::new(),
            ui_lang,
            settings_open: false,
            active_nav,
            vm_inventory: Vec::new(),
            bulk_vm_names: String::new(),
            pending_confirm_stop: false,
            pending_confirm_start: false,
            last_action: String::new(),
            control_addr,
            net_tx,
            net_rx,
            net_busy: false,
            host_connected: false,
            last_capabilities: String::new(),
            last_net_error: String::new(),
        }
    }

    fn persist_snapshot(&self) -> CenterPersist {
        CenterPersist {
            endpoints: self.endpoints.clone(),
            accounts: self.accounts.clone(),
            proxy_labels: self.proxy_labels.clone(),
            last_script_version: self.last_script_version.clone(),
            list_vms_auto_refresh: self.list_vms_auto_refresh,
            list_vms_poll_secs: self.list_vms_poll_secs.max(5),
            discovery_broadcast: self.discovery_broadcast,
            discovery_interval_secs: self.discovery_interval_secs.max(1),
            discovery_udp_port: self.discovery_udp_port,
            ui_lang: self.ui_lang,
            active_nav: self.active_nav,
        }
    }

    fn drain_net_inbox(&mut self) {
        while let Ok(msg) = self.net_rx.try_recv() {
            self.net_busy = false;
            match msg {
                NetUiMsg::Caps { summary } => {
                    self.last_capabilities = summary.clone();
                    if let Some(ep) = self.endpoints.get_mut(self.selected_host) {
                        ep.last_caps = summary;
                    }
                    self.host_connected = true;
                    self.last_net_error.clear();
                    self.last_action = i18n::log_host_responded(self.ui_lang);
                }
                NetUiMsg::VmInventory(vms) => {
                    let n = vms.len();
                    self.vm_inventory = vms;
                    if let Some(ep) = self.endpoints.get_mut(self.selected_host) {
                        ep.last_vm_count = n as u32;
                    }
                    self.last_net_error.clear();
                    self.last_action = i18n::log_list_vms(self.ui_lang, n);
                }
                NetUiMsg::BatchStop {
                    succeeded,
                    failures,
                } => {
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_stop_vm_group(self.ui_lang, succeeded, failures.len());
                    if !failures.is_empty() {
                        self.last_net_error = failures.join("; ");
                    }
                }
                NetUiMsg::BatchStart {
                    succeeded,
                    failures,
                } => {
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_start_vm_group(self.ui_lang, succeeded, failures.len());
                    if !failures.is_empty() {
                        self.last_net_error = failures.join("; ");
                    }
                }
                NetUiMsg::SpoofApply {
                    dry_run,
                    steps,
                    notes,
                } => {
                    self.last_net_error.clear();
                    self.last_action =
                        i18n::log_spoof_apply(self.ui_lang, dry_run, &steps.join(", "), &notes);
                }
                NetUiMsg::GuestAgentReg { vm_name } => {
                    self.last_net_error.clear();
                    self.last_action = i18n::log_guest_reg(self.ui_lang, &vm_name);
                }
                NetUiMsg::Error(e) => {
                    self.last_net_error = e;
                    self.last_action = i18n::log_request_failed(self.ui_lang);
                }
            }
        }
    }

    fn tick_discovery_thread(&mut self) {
        if self.discovery_broadcast && !self.discovery_prev_broadcast {
            let my_gen = self.discovery_gen.fetch_add(1, Ordering::SeqCst) + 1;
            let gen = self.discovery_gen.clone();
            let interval = Duration::from_secs(u64::from(self.discovery_interval_secs.max(1)));
            let port = self.discovery_udp_port;
            let host_control = self.control_addr.clone();
            std::thread::spawn(move || {
                discovery::discovery_udp_loop(my_gen, gen, interval, port, host_control);
            });
        }
        if !self.discovery_broadcast && self.discovery_prev_broadcast {
            self.discovery_gen.fetch_add(1, Ordering::SeqCst);
        }
        self.discovery_prev_broadcast = self.discovery_broadcast;
    }

    fn tick_list_vms_auto_refresh(&mut self, ctx: &egui::Context) {
        if !self.host_connected {
            self.list_vms_poll_accum = 0.0;
        } else if self.list_vms_auto_refresh
            && !self.net_busy
            && !self.control_addr.trim().is_empty()
        {
            self.list_vms_poll_accum += ctx.input(|i| i.unstable_dt);
            if self.list_vms_poll_accum >= self.list_vms_poll_secs.max(5) as f32 {
                self.list_vms_poll_accum = 0.0;
                self.spawn_list_vms();
            }
        }
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        let visuals = ctx.style().visuals.clone();
        let top_fill = visuals.panel_fill;
        // Bottom edge: `show_separator_line` only (matches stroke weight with side nav rule).
        egui::TopBottomPanel::top("status")
            .frame(
                Frame::NONE
                    .fill(top_fill)
                    .inner_margin(Margin::symmetric(22, 12))
                    .stroke(Stroke::NONE)
                    .shadow(card_shadow()),
            )
            .show_separator_line(true)
            .default_height(52.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    self.top_status_bar(ui);
                });
            });
    }

    fn render_side_nav(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let visuals = ctx.style().visuals.clone();
        let nav_fill = visuals.extreme_bg_color;
        let rule_stroke = visuals.widgets.noninteractive.bg_stroke;
        let screen_left = ctx.screen_rect().left();
        let nav_w = if self.header_sep_center_x > 1.0 {
            (self.header_sep_center_x - screen_left + 0.5 * rule_stroke.width)
                .clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH)
        } else {
            SIDEBAR_DEFAULT_WIDTH
        };
        egui::SidePanel::left("nav")
            .exact_width(nav_w)
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(nav_fill)
                    .inner_margin(Margin::symmetric(10, 14))
                    // Right edge is the panel separator only (light gray in theme).
                    .stroke(Stroke::NONE),
            )
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                let tabs = [
                    (NavTab::Monitor, Msg::NavMonitor),
                    (NavTab::Connect, Msg::NavConnect),
                    (NavTab::HostsVms, Msg::NavHostsVms),
                    (NavTab::Spoof, Msg::NavSpoof),
                    (NavTab::Power, Msg::NavPower),
                    (NavTab::Settings, Msg::NavSettings),
                ];
                let mut nav_row = |ui: &mut egui::Ui, tab: NavTab, msg: Msg| {
                    let selected = self.active_nav == tab;
                    let w = ui.available_width();
                    let label = i18n::t(lang, msg);
                    let inactive = ui.visuals().widgets.inactive.text_color();
                    let text = if selected {
                        RichText::new(label).size(14.0).strong().color(ACCENT)
                    } else {
                        RichText::new(label).size(14.0).color(inactive)
                    };
                    let galley = WidgetText::from(text).into_galley(
                        ui,
                        Some(TextWrapMode::Extend),
                        f32::INFINITY,
                        TextStyle::Button,
                    );
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(w, NAV_ITEM_HEIGHT), Sense::click());
                    if ui.is_rect_visible(rect) {
                        let y = rect.center().y - 0.5 * galley.size().y;
                        ui.painter()
                            .galley(egui::pos2(rect.min.x, y), galley, inactive);
                    }
                    if response.clicked() {
                        self.active_nav = tab;
                    }
                };
                for &(tab, msg) in &tabs {
                    nav_row(ui, tab, msg);
                }
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        let central_fill = ctx.style().visuals.window_fill;
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(central_fill)
                    .inner_margin(Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(self.active_nav as u8)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let full_w = ui.available_width();
                        let content_w = full_w.min(CONTENT_MAX_WIDTH);
                        let pad = ((full_w - content_w).max(0.0)) * 0.5;
                        ui.horizontal(|ui| {
                            ui.add_space(pad);
                            ui.vertical(|ui| {
                                ui.set_width(content_w);
                                match self.active_nav {
                                    NavTab::Connect => self.panel_device_management_redirect(ui),
                                    NavTab::Settings => self.panel_settings_host(ui),
                                    NavTab::HostsVms => self.panel_window_management(ui),
                                    NavTab::Monitor => {
                                        let inner = ui.available_width();
                                        let gap = CONTENT_COLUMN_GAP;
                                        let col_w = ((inner - gap).max(0.0)) * 0.5;
                                        ui.horizontal(|ui| {
                                            ui.set_min_width(inner);
                                            ui.vertical(|ui| {
                                                ui.set_width(col_w);
                                                self.panel_tasks(ui);
                                                ui.add_space(PANEL_SPACING);
                                                self.panel_status_board(ui);
                                            });
                                            ui.add_space(gap);
                                            ui.vertical(|ui| {
                                                ui.set_width(col_w);
                                                self.panel_datacenter_stores(ui);
                                            });
                                        });
                                    }
                                    NavTab::Spoof => self.panel_spoof_pipeline(ui, ctx),
                                    NavTab::Power => self.panel_danger(ui, ctx),
                                }
                            });
                            ui.add_space(pad);
                        });
                    });
            });
    }

    fn render_settings_window(&mut self, ctx: &egui::Context) {
        let lang = self.ui_lang;
        let mut close_clicked = false;
        egui::Window::new(i18n::t(lang, Msg::SettingsTitle))
            .open(&mut self.settings_open)
            .collapsible(false)
            .resizable(false)
            .default_pos(ctx.screen_rect().right_top() + egui::vec2(-256.0, 48.0))
            .default_width(248.0)
            .show(ctx, |ui| {
                ui.radio_value(
                    &mut self.ui_lang,
                    UiLang::En,
                    i18n::t(lang, Msg::LangRadioEn),
                );
                ui.radio_value(
                    &mut self.ui_lang,
                    UiLang::Zh,
                    i18n::t(lang, Msg::LangRadioZh),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new(i18n::t(lang, Msg::SettingsMoreLangNote))
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(8.0);
                if ui.button(i18n::t(lang, Msg::SettingsClose)).clicked() {
                    close_clicked = true;
                }
            });
        if close_clicked {
            self.settings_open = false;
        }
    }
}

impl eframe::App for CenterApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(&self.persist_snapshot()) {
            storage.set_string(PERSIST_KEY, json);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_net_inbox();
        self.tick_discovery_thread();
        self.tick_list_vms_auto_refresh(ctx);
        self.render_top_panel(ctx);
        self.render_side_nav(ctx);
        self.render_central_panel(ctx);
        self.render_settings_window(ctx);
    }
}
