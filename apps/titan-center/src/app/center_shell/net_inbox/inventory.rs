use crate::app::CenterApp;
use crate::app::i18n;
use crate::app::net::NetUiMsg;

impl CenterApp {
    pub(super) fn try_net_inventory_ops(&mut self, msg: &NetUiMsg) -> Option<bool> {
        self.try_net_inventory_caps_and_vm(msg)
    }

    fn try_net_inventory_caps_and_vm(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::Caps { summary } => {
                self.on_net_caps(summary.clone());
                Some(false)
            }
            NetUiMsg::VmInventory(vms) => {
                self.on_net_vm_inventory(vms.clone());
                Some(false)
            }
            _ => None,
        }
    }

    fn on_net_caps(&mut self, summary: String) {
        self.net_busy = false;
        self.upsert_endpoint_after_caps(summary);
        let control_addr = self.control_addr.clone();
        self.mark_command_ready_for_addr(&control_addr, true);
        self.last_net_error.clear();
        self.last_action = i18n::log_host_responded(self.ui_lang);
        self.spawn_telemetry_reader();
        self.spawn_ui_lang_push_to_host_control_addr(&self.control_addr);
        self.ctx.request_repaint();
    }

    fn on_net_vm_inventory(&mut self, vms: Vec<titan_common::VmBrief>) {
        self.net_busy = false;
        let n = vms.len();
        let key = Self::endpoint_addr_key(&self.control_addr);
        if !key.is_empty() {
            let st = self.fleet_by_endpoint.entry(key).or_default();
            st.vms = vms.clone();
        }
        self.vm_inventory = vms;
        if let Some(ep) = self.endpoint_mut_for_control_addr() {
            ep.last_vm_count = n as u32;
        }
        self.last_net_error.clear();
        self.last_action = i18n::log_list_vms(self.ui_lang, n);
    }
}
