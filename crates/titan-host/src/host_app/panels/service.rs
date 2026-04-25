use crate::host_app::model::{AgentBindingRow, HostApp};

impl HostApp {
    pub(crate) fn panel_service(&mut self, ui: &mut egui::Ui) {
        self.panel_service_env_hint(ui);
        self.panel_service_listen_row(ui);
        self.panel_service_announce_block(ui);
        self.panel_service_public_and_label(ui);
        ui.add_space(8.0);
        ui.heading("Agent 绑定 (VM 名 → 地址)");
        self.panel_service_agent_bind_row(ui);
        self.panel_service_agent_scroll(ui);
        ui.add_space(12.0);
        self.panel_service_save_and_status(ui);
    }

    fn panel_service_env_hint(&self, ui: &mut egui::Ui) {
        if let Some(ref h) = self.env_listen_hint {
            ui.label(egui::RichText::new(h).weak());
            ui.add_space(4.0);
        }
    }

    fn panel_service_listen_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("监听地址");
            ui.text_edit_singleline(&mut self.persist.listen);
        });
    }

    fn panel_service_announce_block(&mut self, ui: &mut egui::Ui) {
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
        self.panel_service_announce_periodic(ui);
    }

    fn panel_service_announce_periodic(&mut self, ui: &mut egui::Ui) {
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
    }

    fn panel_service_public_and_label(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("公网/展示地址覆盖");
            ui.text_edit_singleline(&mut self.persist.public_addr_override);
        });
        ui.horizontal(|ui| {
            ui.label("主机标签覆盖");
            ui.text_edit_singleline(&mut self.persist.label_override);
        });
    }

    fn panel_service_agent_bind_row(&mut self, ui: &mut egui::Ui) {
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
    }

    fn panel_service_agent_scroll(&mut self, ui: &mut egui::Ui) {
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
    }

    fn panel_service_save_and_status(&mut self, ui: &mut egui::Ui) {
        if ui.button("保存并重启控制面").clicked() {
            self.start_serve();
        }
        if !self.status_line.is_empty() {
            ui.add_space(4.0);
            ui.label(&self.status_line);
        }
    }
}
