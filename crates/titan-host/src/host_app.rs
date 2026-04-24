//! egui shell: service settings, agent bindings, and batch VM provisioning (no hand-edited TOML).

use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;
use serde::{Deserialize, Serialize};
use titan_common::{
    VmProvisionPlan, DEFAULT_CENTER_POLL_UDP_PORT, DEFAULT_CENTER_REGISTER_UDP_PORT,
};
use titan_vmm::hyperv::AgentBindingTable;
use tokio::sync::watch;

use crate::batch::run_provision_plans;
use crate::config::{expand_vm_plans, VmGroup};
use crate::host_font;
use crate::serve::{run_serve, AgentBindingsSpec, HostAnnounceConfig};

const PERSIST_KEY: &str = "titan_host_ui_v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBindingRow {
    pub vm_name: String,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostUiPersist {
    pub listen: String,
    pub announce_enabled: bool,
    /// Seconds between periodic UDP announces; `None` = disabled.
    pub announce_periodic_secs: Option<u64>,
    pub center_register_udp_port: u16,
    pub center_poll_listen_port: u16,
    pub public_addr_override: String,
    pub label_override: String,
    pub agent_rows: Vec<AgentBindingRow>,
    pub batch_timeout_secs: u64,
    pub batch_fail_fast: bool,
    pub batch_vm: Vec<VmProvisionPlan>,
    pub batch_vm_group: Vec<VmGroup>,
}

impl Default for HostUiPersist {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:7788".into(),
            announce_enabled: true,
            announce_periodic_secs: None,
            center_register_udp_port: DEFAULT_CENTER_REGISTER_UDP_PORT,
            center_poll_listen_port: DEFAULT_CENTER_POLL_UDP_PORT,
            public_addr_override: String::new(),
            label_override: String::new(),
            agent_rows: Vec::new(),
            batch_timeout_secs: 600,
            batch_fail_fast: false,
            batch_vm: Vec::new(),
            batch_vm_group: Vec::new(),
        }
    }
}

impl HostUiPersist {
    fn parse_listen(&self) -> Result<SocketAddr, String> {
        self.listen
            .trim()
            .parse()
            .map_err(|e| format!("监听地址无效: {e}"))
    }

    fn to_announce(&self) -> HostAnnounceConfig {
        HostAnnounceConfig {
            enabled: self.announce_enabled,
            periodic_interval: self
                .announce_periodic_secs
                .filter(|&s| s > 0)
                .map(Duration::from_secs),
            center_register_udp_port: self.center_register_udp_port,
            center_poll_listen_port: self.center_poll_listen_port,
            public_addr_override: {
                let s = self.public_addr_override.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            },
            label_override: {
                let s = self.label_override.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            },
        }
    }

    fn bindings_spec(&self) -> Result<AgentBindingsSpec, String> {
        let table = AgentBindingTable::new();
        for row in &self.agent_rows {
            let vm = row.vm_name.trim();
            if vm.is_empty() {
                continue;
            }
            let addr: SocketAddr = row
                .addr
                .trim()
                .parse()
                .map_err(|e| format!("{} 的地址无效: {e}", row.vm_name))?;
            table.insert(vm.to_string(), addr);
        }
        Ok(AgentBindingsSpec::Inline {
            agents: Arc::new(table),
            notice: String::new(),
        })
    }
}

struct ServeRun {
    shutdown_tx: watch::Sender<bool>,
    join: JoinHandle<()>,
}

impl ServeRun {
    fn stop(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.join.join();
    }
}

pub struct HostApp {
    ctx: egui::Context,
    really_quitting: bool,
    hidden_to_tray: bool,
    _tray: Option<titan_tray::TrayIcon>,
    serve_run: Option<ServeRun>,
    persist: HostUiPersist,
    active_tab: usize,
    status_line: String,
    provision_log: Vec<String>,
    provision_rx: Option<mpsc::Receiver<String>>,
    env_listen_hint: Option<String>,
    binding_vm: String,
    binding_addr: String,
    /// First `update` tick starts serve once (invalid listen → user fixes and clicks restart).
    initial_serve_attempted: bool,
    /// One-shot: bring the native window to front after eframe's initial `with_visible(false)` bootstrap.
    boot_window_focus_once: bool,
}

impl HostApp {
    /// `initial_tray`: build with [`titan_tray::build_host_tray_icon`] in the `eframe::run_native` closure
    /// **before** constructing the app (matches tray-icon's egui example; avoids macOS first-frame ordering issues).
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_tray: Option<titan_tray::TrayIcon>,
    ) -> Self {
        host_font::install_cjk_fonts(&cc.egui_ctx);

        let json_opt = cc.storage.and_then(|s| s.get_string(PERSIST_KEY));
        let mut persist: HostUiPersist = json_opt
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let mut env_listen_hint = None;
        if let Ok(s) = std::env::var("TITAN_HOST_LISTEN") {
            if s.parse::<SocketAddr>().is_ok() {
                persist.listen = s;
                env_listen_hint = Some("已应用环境变量 TITAN_HOST_LISTEN".into());
            }
        }

        Self {
            ctx: cc.egui_ctx.clone(),
            really_quitting: false,
            hidden_to_tray: false,
            _tray: initial_tray,
            serve_run: None,
            persist,
            active_tab: 0,
            status_line: String::new(),
            provision_log: Vec::new(),
            provision_rx: None,
            env_listen_hint,
            binding_vm: String::new(),
            binding_addr: String::new(),
            initial_serve_attempted: false,
            boot_window_focus_once: false,
        }
    }

    fn start_serve(&mut self) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }

        let listen = match self.persist.parse_listen() {
            Ok(a) => a,
            Err(e) => {
                self.status_line = e;
                return;
            }
        };

        let spec = match self.persist.bindings_spec() {
            Ok(s) => s,
            Err(e) => {
                self.status_line = e;
                return;
            }
        };

        let announce = self.persist.to_announce();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let join = std::thread::Builder::new()
            .name("titan-host-serve".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("tokio runtime: {e}");
                        return;
                    }
                };
                if let Err(e) = rt.block_on(run_serve(listen, spec, announce, shutdown_rx)) {
                    tracing::warn!(error = %e, "serve thread ended with error");
                } else {
                    tracing::info!("serve thread ended");
                }
            })
            .expect("spawn serve thread");

        self.serve_run = Some(ServeRun { shutdown_tx, join });
        self.status_line = format!("控制面已监听 {}", self.persist.listen);
    }

    fn drain_provision_log(&mut self) {
        let Some(rx) = self.provision_rx.as_ref() else {
            return;
        };
        while let Ok(line) = rx.try_recv() {
            self.provision_log.push(line);
            if self.provision_log.len() > 400 {
                self.provision_log
                    .drain(0..self.provision_log.len().saturating_sub(300));
            }
        }
    }

    fn run_provision_clicked(&mut self, dry_run: bool) {
        let plans = match expand_vm_plans(&self.persist.batch_vm, &self.persist.batch_vm_group) {
            Ok(p) => p,
            Err(e) => {
                self.status_line = e.to_string();
                return;
            }
        };

        if plans.is_empty() {
            self.status_line = "没有可创建的虚拟机：请添加「显式 VM」或「VM 组」".into();
            return;
        }

        let timeout = Duration::from_secs(self.persist.batch_timeout_secs.max(1));
        let fail_fast = self.persist.batch_fail_fast;
        let (tx, rx) = mpsc::channel();
        self.provision_rx = Some(rx);
        self.provision_log.clear();
        let _ = tx.send(format!(
            "开始{} — 共 {} 台",
            if dry_run {
                "预检 (dry-run)"
            } else {
                "创建"
            },
            plans.len()
        ));

        std::thread::Builder::new()
            .name("titan-host-provision".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(format!("tokio runtime: {e}"));
                        return;
                    }
                };
                let res = rt.block_on(run_provision_plans(plans, timeout, fail_fast, dry_run));
                let msg = match res {
                    Ok(()) => "批量任务结束".into(),
                    Err(e) => format!("批量失败: {e}"),
                };
                let _ = tx.send(msg);
            })
            .expect("spawn provision");

        self.ctx.request_repaint();
    }

    fn panel_service(&mut self, ui: &mut egui::Ui) {
        if let Some(ref h) = self.env_listen_hint {
            ui.label(egui::RichText::new(h).weak());
            ui.add_space(4.0);
        }

        ui.horizontal(|ui| {
            ui.label("监听地址");
            ui.text_edit_singleline(&mut self.persist.listen);
        });

        ui.checkbox(&mut self.persist.announce_enabled, "启用 LAN 注册 / 应答");
        ui.horizontal(|ui| {
            ui.label("轮询 UDP 端口");
            ui.add(
                egui::DragValue::new(&mut self.persist.center_poll_listen_port)
                    .speed(1.0)
                    .range(1..=65535),
            );
        });
        ui.horizontal(|ui| {
            ui.label("注册 UDP 端口");
            ui.add(
                egui::DragValue::new(&mut self.persist.center_register_udp_port)
                    .speed(1.0)
                    .range(1..=65535),
            );
        });

        let mut periodic = self.persist.announce_periodic_secs.unwrap_or(0);
        ui.horizontal(|ui| {
            ui.label("周期广播间隔 (秒，0=关闭)");
            if ui
                .add(
                    egui::DragValue::new(&mut periodic)
                        .speed(1.0)
                        .range(0..=86400),
                )
                .changed()
            {
                self.persist.announce_periodic_secs =
                    if periodic > 0 { Some(periodic) } else { None };
            }
        });

        ui.horizontal(|ui| {
            ui.label("公网/展示地址覆盖");
            ui.text_edit_singleline(&mut self.persist.public_addr_override);
        });
        ui.horizontal(|ui| {
            ui.label("主机标签覆盖");
            ui.text_edit_singleline(&mut self.persist.label_override);
        });

        ui.add_space(8.0);
        ui.heading("Agent 绑定 (VM 名 → 地址)");
        ui.horizontal(|ui| {
            ui.label("VM");
            ui.text_edit_singleline(&mut self.binding_vm);
            ui.label("地址");
            ui.text_edit_singleline(&mut self.binding_addr);
            if ui.button("添加").clicked() {
                self.persist.agent_rows.push(AgentBindingRow {
                    vm_name: self.binding_vm.trim().to_string(),
                    addr: self.binding_addr.trim().to_string(),
                });
                self.binding_vm.clear();
                self.binding_addr.clear();
            }
        });

        let mut remove_idx: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("host_panel_service_agent_rows")
            .max_height(180.0)
            .show(ui, |ui| {
                for (i, row) in self.persist.agent_rows.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut row.vm_name);
                        ui.text_edit_singleline(&mut row.addr);
                        if ui.button("删").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                }
            });
        if let Some(i) = remove_idx {
            self.persist.agent_rows.remove(i);
        }

        ui.add_space(12.0);
        if ui.button("保存并重启控制面").clicked() {
            self.start_serve();
        }
        if !self.status_line.is_empty() {
            ui.add_space(4.0);
            ui.label(&self.status_line);
        }
    }

    fn panel_batch(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("每台 VM 超时 (秒)");
            ui.add(
                egui::DragValue::new(&mut self.persist.batch_timeout_secs)
                    .speed(10.0)
                    .range(1..=86400),
            );
        });
        ui.checkbox(&mut self.persist.batch_fail_fast, "遇错即停 (fail-fast)");

        ui.add_space(8.0);
        ui.heading("显式 VM (VmProvisionPlan)");
        let mut remove_vm: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("host_panel_batch_vm_plans")
            .max_height(200.0)
            .show(ui, |ui| {
                for (i, p) in self.persist.batch_vm.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("名称");
                            ui.text_edit_singleline(&mut p.vm_name);
                            if ui.button("删除").clicked() {
                                remove_vm = Some(i);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("父 VHDX");
                            ui.text_edit_singleline(&mut p.parent_vhdx);
                        });
                        ui.horizontal(|ui| {
                            ui.label("差分目录");
                            ui.text_edit_singleline(&mut p.diff_dir);
                        });
                        ui.horizontal(|ui| {
                            ui.label("内存 (字节)");
                            ui.add(egui::DragValue::new(&mut p.memory_bytes).speed(256.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("代数");
                            ui.add(
                                egui::DragValue::new(&mut p.generation)
                                    .speed(1.0)
                                    .range(1..=2),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("交换机 (空=无)");
                            let mut sw = p.switch_name.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut sw).changed() {
                                let t = sw.trim();
                                p.switch_name = if t.is_empty() {
                                    None
                                } else {
                                    Some(t.to_string())
                                };
                            }
                        });
                        ui.checkbox(&mut p.auto_start_after_provision, "创建后自动启动");
                        ui.horizontal(|ui| {
                            ui.label("GPU 实例路径 (可选)");
                            let mut g = p.gpu_partition_instance_path.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut g).changed() {
                                let t = g.trim();
                                p.gpu_partition_instance_path = if t.is_empty() {
                                    None
                                } else {
                                    Some(t.to_string())
                                };
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut p.spoof.dynamic_mac, "动态 MAC");
                            ui.checkbox(&mut p.spoof.disable_checkpoints, "禁用检查点");
                        });
                        ui.horizontal(|ui| {
                            ui.label("CPU 数 (0=默认)");
                            let mut n = p.spoof.processor_count.unwrap_or(0);
                            if ui
                                .add(egui::DragValue::new(&mut n).speed(1.0).range(0..=256))
                                .changed()
                            {
                                p.spoof.processor_count = if n == 0 { None } else { Some(n) };
                            }
                        });
                    });
                }
            });
        if let Some(i) = remove_vm {
            self.persist.batch_vm.remove(i);
        }
        if ui.button("添加显式 VM").clicked() {
            self.persist.batch_vm.push(VmProvisionPlan {
                parent_vhdx: String::new(),
                diff_dir: String::new(),
                vm_name: format!("vm-{}", self.persist.batch_vm.len()),
                memory_bytes: 2 * 1024 * 1024 * 1024,
                generation: 2,
                switch_name: None,
                gpu_partition_instance_path: None,
                auto_start_after_provision: true,
                spoof: Default::default(),
                identity: Default::default(),
            });
        }

        ui.add_space(8.0);
        ui.heading("VM 组模板 (vm_group)");
        let mut remove_g: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("host_panel_batch_vm_groups")
            .max_height(160.0)
            .show(ui, |ui| {
                for (i, g) in self.persist.batch_vm_group.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("前缀");
                            ui.text_edit_singleline(&mut g.name_prefix);
                            ui.label("数量");
                            ui.add(egui::DragValue::new(&mut g.count).speed(1.0).range(0..=64));
                            if ui.button("删除组").clicked() {
                                remove_g = Some(i);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("父 VHDX");
                            ui.text_edit_singleline(&mut g.parent_vhdx);
                        });
                        ui.horizontal(|ui| {
                            ui.label("差分目录");
                            ui.text_edit_singleline(&mut g.diff_dir);
                        });
                        ui.horizontal(|ui| {
                            ui.label("内存");
                            ui.add(egui::DragValue::new(&mut g.memory_bytes).speed(256.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("代数");
                            ui.add(
                                egui::DragValue::new(&mut g.generation)
                                    .speed(1.0)
                                    .range(1..=2),
                            );
                        });
                    });
                }
            });
        if let Some(i) = remove_g {
            self.persist.batch_vm_group.remove(i);
        }
        if ui.button("添加 VM 组").clicked() {
            self.persist.batch_vm_group.push(VmGroup {
                parent_vhdx: String::new(),
                diff_dir: String::new(),
                name_prefix: "game-".into(),
                count: 1,
                memory_bytes: 1024 * 1024 * 1024,
                generation: 2,
                switch_name: None,
                gpu_partition_instance_path: None,
                auto_start_after_provision: true,
                spoof: Default::default(),
                identity: Default::default(),
            });
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("预检 (dry-run)").clicked() {
                self.run_provision_clicked(true);
            }
            if ui.button("开始创建").clicked() {
                self.run_provision_clicked(false);
            }
        });

        ui.add_space(6.0);
        ui.label("日志");
        egui::ScrollArea::vertical()
            .id_salt("host_panel_batch_provision_log")
            .max_height(200.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.provision_log {
                    ui.label(egui::RichText::new(line).monospace());
                }
            });
    }
}

impl eframe::App for HostApp {
    /// Do not persist egui memory: otherwise a previous "close to tray" session restores
    /// `ViewportCommand::Visible(false)` and the main window can stay hidden on launch.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.really_quitting || raw_input.viewport_id != egui::ViewportId::ROOT {
            return;
        }
        if !raw_input.viewport().close_requested() {
            return;
        }
        if let Some(vp) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            vp.events.retain(|e| *e != egui::ViewportEvent::Close);
        }
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::Visible(false),
        );
        ctx.request_repaint_after_for(
            std::time::Duration::from_millis(250),
            egui::ViewportId::ROOT,
        );
        self.hidden_to_tray = true;
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(&self.persist) {
            storage.set_string(PERSIST_KEY, json);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(r) = self.serve_run.take() {
            r.stop();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.boot_window_focus_once && !self.hidden_to_tray {
            self.boot_window_focus_once = true;
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
        }

        if titan_tray::poll_tray_for_egui_product(
            ctx,
            &mut self.really_quitting,
            titan_tray::DesktopProduct::Host,
        ) {
            self.hidden_to_tray = false;
        }
        if self.hidden_to_tray {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }

        self.drain_provision_log();

        if !self.initial_serve_attempted {
            self.initial_serve_attempted = true;
            self.start_serve();
        }

        egui::TopBottomPanel::top("host_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Titan Host");
                ui.separator();
                if ui.selectable_label(self.active_tab == 0, "服务").clicked() {
                    self.active_tab = 0;
                }
                if ui
                    .selectable_label(self.active_tab == 1, "批量创建")
                    .clicked()
                {
                    self.active_tab = 1;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            0 => self.panel_service(ui),
            1 => self.panel_batch(ui),
            _ => {}
        });
    }
}
