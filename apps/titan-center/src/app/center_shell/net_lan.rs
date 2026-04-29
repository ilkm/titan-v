//! LAN host announce merge for [`NetUiMsg::HostAnnounced`](crate::app::net::NetUiMsg).

use crate::app::CenterApp;
use crate::app::i18n;
use crate::app::persist_data::HostEndpoint;

impl CenterApp {
    pub(crate) fn apply_net_host_announced(
        &mut self,
        control_addr: String,
        label: String,
        device_id: String,
    ) {
        let addr = Self::endpoint_addr_key(&control_addr);
        if addr.is_empty() {
            return;
        }
        let id_from_host = device_id.trim();
        let resolved_label = resolve_announced_label(&label, &addr);
        let new_addr = control_addr.trim().to_string();
        let new_key = Self::endpoint_addr_key(&new_addr);
        let lone_legacy = self.lone_legacy_endpoint_index();
        if !id_from_host.is_empty() {
            self.merge_announced_nonempty_device_id(
                &addr,
                id_from_host,
                &new_addr,
                &new_key,
                &resolved_label,
                lone_legacy,
            );
        } else {
            self.merge_announced_empty_device_id(&addr, &resolved_label);
        }
        self.finish_net_host_announced_merge(&resolved_label, &addr);
        self.spawn_ui_lang_push_to_host_control_addr(&control_addr);
    }

    fn finish_net_host_announced_merge(&mut self, resolved_label: &str, addr: &str) {
        self.persist_registered_devices();
        self.last_net_error.clear();
        self.last_action = i18n::log_lan_host_announced(self.ui_lang, resolved_label, addr);
        self.ctx.request_repaint();
    }

    fn lone_legacy_endpoint_index(&self) -> Option<usize> {
        let hits: Vec<usize> = self
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.device_id.trim().is_empty()
                    || e.device_id == HostEndpoint::legacy_device_id_for_addr(&e.addr)
            })
            .map(|(i, _)| i)
            .collect();
        if hits.len() == 1 { Some(hits[0]) } else { None }
    }

    fn merge_announced_nonempty_device_id(
        &mut self,
        addr: &str,
        id_from_host: &str,
        new_addr: &str,
        new_key: &str,
        resolved_label: &str,
        lone_legacy: Option<usize>,
    ) {
        if self.try_rebind_announced_by_device_id(id_from_host, new_addr, new_key, resolved_label) {
            return;
        }
        if let Some(pos) = lone_legacy {
            self.rebind_announced_endpoint(pos, new_addr, new_key, resolved_label);
            self.endpoints[pos].device_id = id_from_host.to_string();
            return;
        }
        self.push_announced_new_endpoint(addr, id_from_host, resolved_label);
    }

    fn try_rebind_announced_by_device_id(
        &mut self,
        id_from_host: &str,
        new_addr: &str,
        new_key: &str,
        resolved_label: &str,
    ) -> bool {
        let Some(pos) = self
            .endpoints
            .iter()
            .position(|e| e.device_id == id_from_host)
        else {
            return false;
        };
        self.rebind_announced_endpoint(pos, new_addr, new_key, resolved_label);
        true
    }

    fn push_announced_new_endpoint(
        &mut self,
        addr: &str,
        id_from_host: &str,
        resolved_label: &str,
    ) {
        self.endpoints.push(HostEndpoint {
            label: resolved_label.to_string(),
            addr: addr.to_string(),
            device_id: id_from_host.to_string(),
            remark: String::new(),
            last_caps: String::new(),
            last_vm_count: 0,
            last_known_online: false,
        });
    }

    fn merge_announced_empty_device_id(&mut self, addr: &str, resolved_label: &str) {
        if let Some(ep) = self
            .endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == addr)
        {
            if ep.label != resolved_label {
                ep.label = resolved_label.to_string();
            }
            return;
        }
        self.endpoints.push(HostEndpoint {
            label: resolved_label.to_string(),
            addr: addr.to_string(),
            device_id: HostEndpoint::legacy_device_id_for_addr(addr),
            remark: String::new(),
            last_caps: String::new(),
            last_vm_count: 0,
            last_known_online: false,
        });
    }

    fn rebind_announced_endpoint(
        &mut self,
        pos: usize,
        new_addr: &str,
        new_key: &str,
        resolved_label: &str,
    ) {
        let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
        if old_key != *new_key {
            self.stop_telemetry_reader_for_key(&old_key);
            self.remap_host_caches_addr_key(&old_key, new_key);
            if old_key == Self::endpoint_addr_key(&self.control_addr) {
                self.control_addr = new_addr.to_string();
                self.command_ready = false;
                self.host_connected = false;
                self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
            }
        }
        let ep = &mut self.endpoints[pos];
        ep.addr = new_addr.to_string();
        if ep.label != resolved_label {
            ep.label = resolved_label.to_string();
        }
    }
}

fn resolve_announced_label(label: &str, addr: &str) -> String {
    if label.trim().is_empty() {
        format!("host-{}", addr.replace([':', '.'], "-"))
    } else {
        label.trim().to_string()
    }
}
