use titan_common::VmProvisionPlan;

use crate::config::VmGroup;
use crate::host_app::model::HostApp;

impl HostApp {
    pub(crate) fn panel_batch(&mut self, ui: &mut egui::Ui) {
        self.panel_batch_timeout_rows(ui);
        ui.add_space(8.0);
        ui.heading("显式 VM (VmProvisionPlan)");
        self.panel_batch_vm_plans_block(ui);
        ui.add_space(8.0);
        ui.heading("VM 组模板 (vm_group)");
        self.panel_batch_vm_groups_block(ui);
        ui.add_space(8.0);
        self.panel_batch_provision_buttons(ui);
        ui.add_space(6.0);
        self.panel_batch_provision_log(ui);
    }

    fn panel_batch_timeout_rows(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("每台 VM 超时 (秒)");
            ui.add(
                egui::DragValue::new(&mut self.persist.batch_timeout_secs)
                    .speed(10.0)
                    .range(1..=86400),
            );
        });
        ui.checkbox(&mut self.persist.batch_fail_fast, "遇错即停 (fail-fast)");
    }

    fn panel_batch_vm_plans_block(&mut self, ui: &mut egui::Ui) {
        let mut remove_vm: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("host_panel_batch_vm_plans")
            .max_height(200.0)
            .show(ui, |ui| {
                for (i, p) in self.persist.batch_vm.iter_mut().enumerate() {
                    ui.group(|ui| {
                        Self::panel_batch_edit_vm_plan_group(ui, p, i, &mut remove_vm);
                    });
                }
            });
        if let Some(i) = remove_vm {
            self.persist.batch_vm.remove(i);
        }
        if ui.button("添加显式 VM").clicked() {
            self.panel_batch_push_blank_vm_plan();
        }
    }

    fn panel_batch_push_blank_vm_plan(&mut self) {
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

    fn panel_batch_edit_vm_plan_group(
        ui: &mut egui::Ui,
        p: &mut VmProvisionPlan,
        i: usize,
        remove_vm: &mut Option<usize>,
    ) {
        Self::panel_batch_vm_plan_name_and_paths(ui, p, i, remove_vm);
        Self::panel_batch_vm_plan_memory_and_gen(ui, p);
        Self::panel_batch_vm_plan_switch_and_gpu(ui, p);
        Self::panel_batch_vm_plan_spoof_rows(ui, p);
    }

    fn panel_batch_vm_plan_name_and_paths(
        ui: &mut egui::Ui,
        p: &mut VmProvisionPlan,
        i: usize,
        remove_vm: &mut Option<usize>,
    ) {
        ui.horizontal(|ui| {
            ui.label("名称");
            ui.text_edit_singleline(&mut p.vm_name);
            if ui.button("删除").clicked() {
                *remove_vm = Some(i);
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
    }

    fn panel_batch_vm_plan_memory_and_gen(ui: &mut egui::Ui, p: &mut VmProvisionPlan) {
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
    }

    fn panel_batch_vm_plan_switch_and_gpu(ui: &mut egui::Ui, p: &mut VmProvisionPlan) {
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
    }

    fn panel_batch_vm_plan_spoof_rows(ui: &mut egui::Ui, p: &mut VmProvisionPlan) {
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
    }

    fn panel_batch_vm_groups_block(&mut self, ui: &mut egui::Ui) {
        let mut remove_g: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("host_panel_batch_vm_groups")
            .max_height(160.0)
            .show(ui, |ui| {
                for (i, g) in self.persist.batch_vm_group.iter_mut().enumerate() {
                    ui.group(|ui| {
                        Self::panel_batch_edit_vm_group(ui, g, i, &mut remove_g);
                    });
                }
            });
        if let Some(i) = remove_g {
            self.persist.batch_vm_group.remove(i);
        }
        if ui.button("添加 VM 组").clicked() {
            self.panel_batch_push_default_vm_group();
        }
    }

    fn panel_batch_push_default_vm_group(&mut self) {
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

    fn panel_batch_edit_vm_group(
        ui: &mut egui::Ui,
        g: &mut VmGroup,
        i: usize,
        remove_g: &mut Option<usize>,
    ) {
        ui.horizontal(|ui| {
            ui.label("前缀");
            ui.text_edit_singleline(&mut g.name_prefix);
            ui.label("数量");
            ui.add(egui::DragValue::new(&mut g.count).speed(1.0).range(0..=64));
            if ui.button("删除组").clicked() {
                *remove_g = Some(i);
            }
        });
        Self::panel_batch_vm_group_paths_and_sizing(ui, g);
    }

    fn panel_batch_vm_group_paths_and_sizing(ui: &mut egui::Ui, g: &mut VmGroup) {
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
    }

    fn panel_batch_provision_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("预检 (dry-run)").clicked() {
                self.run_provision_clicked(true);
            }
            if ui.button("开始创建").clicked() {
                self.run_provision_clicked(false);
            }
        });
    }

    fn panel_batch_provision_log(&self, ui: &mut egui::Ui) {
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
