use std::time::{Duration, Instant};

use crate::app::CenterApp;
use crate::app::i18n::{self, Msg};
use crate::app::persist_data::HostEndpoint;

impl CenterApp {
    pub(crate) fn merge_add_host_after_verify(
        &mut self,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        let new_key = Self::endpoint_addr_key(&addr);
        if let Some(pos) = self.endpoints.iter().position(|e| e.device_id == device_id) {
            self.merge_add_host_existing_device(pos, &addr, &new_key, caps_summary);
            return;
        }
        if let Some(pos) = self.merge_add_host_legacy_row_index(&new_key) {
            self.apply_merge_add_host_legacy_row(pos, addr, device_id, caps_summary);
            return;
        }
        self.push_new_host_endpoint_after_verify(addr, device_id, caps_summary);
    }

    fn merge_add_host_legacy_row_index(&self, new_key: &str) -> Option<usize> {
        self.endpoints.iter().position(|e| {
            Self::endpoint_addr_key(&e.addr) == new_key
                && (e.device_id.trim().is_empty()
                    || e.device_id == HostEndpoint::legacy_device_id_for_addr(&e.addr))
        })
    }

    fn apply_merge_add_host_legacy_row(
        &mut self,
        pos: usize,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        let ep = &mut self.endpoints[pos];
        ep.device_id = device_id;
        ep.addr = addr;
        ep.last_caps = caps_summary;
        ep.last_known_online = true;
    }

    fn push_new_host_endpoint_after_verify(
        &mut self,
        addr: String,
        device_id: String,
        caps_summary: String,
    ) {
        self.endpoints.push(HostEndpoint {
            label: format!("host-{}", self.endpoints.len() + 1),
            addr,
            device_id,
            remark: String::new(),
            last_caps: caps_summary,
            last_vm_count: 0,
            last_known_online: true,
        });
    }

    fn merge_add_host_existing_device(
        &mut self,
        pos: usize,
        addr: &str,
        new_key: &str,
        caps_summary: String,
    ) {
        let old_key = Self::endpoint_addr_key(&self.endpoints[pos].addr);
        if old_key != *new_key {
            self.stop_telemetry_reader_for_key(&old_key);
            self.remap_host_caches_addr_key(&old_key, new_key);
            if old_key == Self::endpoint_addr_key(&self.control_addr) {
                self.control_addr = addr.to_string();
                self.command_ready = false;
                self.host_connected = false;
                self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
            }
        }
        let ep = &mut self.endpoints[pos];
        ep.addr = addr.to_string();
        ep.last_caps = caps_summary;
        ep.last_known_online = true;
    }

    pub(crate) fn invalidate_add_host_probe(&mut self) {
        self.add_host_verify_session = self.add_host_verify_session.wrapping_add(1);
        self.add_host_verify_busy = false;
        self.add_host_verify_deadline = None;
    }

    pub(crate) fn tick_add_host_verify_watchdog(&mut self, ctx: &egui::Context) {
        if !self.add_host_verify_busy {
            return;
        }
        let Some(dl) = self.add_host_verify_deadline else {
            return;
        };
        let now = Instant::now();
        if now < dl {
            let wait = dl
                .saturating_duration_since(now)
                .min(Duration::from_millis(400));
            ctx.request_repaint_after(wait);
            return;
        }
        self.invalidate_add_host_probe();
        self.ui_toast_text = i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
        self.ui_toast_until = Some(ctx.input(|i| i.time) + 3.8);
        ctx.request_repaint();
    }
}
