//! `NetUiMsg` dispatch from the UI-thread inbox (`drain_net_inbox`).

use std::time::Instant;

use super::i18n::{self, Msg};
use super::net_msg::NetUiMsg;
use super::CenterApp;

impl CenterApp {
    pub(crate) fn drain_net_inbox(&mut self) {
        while let Ok(msg) = self.net_rx.try_recv() {
            if self.dispatch_net_ui_msg(msg) {
                continue;
            }
        }
    }

    /// Returns `true` when the outer `while` should `continue` (stale / ignored message).
    fn dispatch_net_ui_msg(&mut self, msg: NetUiMsg) -> bool {
        if let Some(c) = self.try_net_inventory_ops(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_add_host_verify_only(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_host_resources_desktop(&msg) {
            return c;
        }
        if let Some(c) = self.try_net_reachability_telemetry_ops(&msg) {
            return c;
        }
        self.dispatch_net_fleet_and_error(msg)
    }

    fn try_net_inventory_ops(&mut self, msg: &NetUiMsg) -> Option<bool> {
        self.try_net_inventory_caps_and_vm(msg)
            .or_else(|| self.try_net_inventory_batches_and_spoof(msg))
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

    fn try_net_inventory_batches_and_spoof(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::BatchStop {
                succeeded,
                failures,
            } => {
                self.on_net_batch_stop(*succeeded, failures.clone());
                Some(false)
            }
            NetUiMsg::BatchStart {
                succeeded,
                failures,
            } => {
                self.on_net_batch_start(*succeeded, failures.clone());
                Some(false)
            }
            NetUiMsg::SpoofApply {
                dry_run,
                steps,
                notes,
            } => {
                self.on_net_spoof_apply(*dry_run, steps.clone(), notes.clone());
                Some(false)
            }
            _ => None,
        }
    }

    fn try_net_add_host_verify_only(&mut self, msg: &NetUiMsg) -> Option<bool> {
        let NetUiMsg::AddHostVerifyDone {
            session_id,
            addr,
            ok,
            device_id,
            caps_summary,
            error,
        } = msg
        else {
            return None;
        };
        Some(self.on_net_add_host_verify_done(
            *session_id,
            addr.clone(),
            *ok,
            device_id.clone(),
            caps_summary.clone(),
            error.clone(),
        ))
    }

    fn try_net_host_resources_desktop(&mut self, msg: &NetUiMsg) -> Option<bool> {
        self.try_net_host_announced_only(msg)
            .or_else(|| self.try_net_host_resources_payload(msg))
    }

    fn try_net_host_announced_only(&mut self, msg: &NetUiMsg) -> Option<bool> {
        let NetUiMsg::HostAnnounced {
            control_addr,
            label,
            device_id,
        } = msg
        else {
            return None;
        };
        self.apply_net_host_announced(control_addr.clone(), label.clone(), device_id.clone());
        Some(false)
    }

    fn try_net_host_resources_payload(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::HostResources {
                control_addr,
                stats,
            } => {
                self.on_net_host_resources(control_addr.clone(), stats.clone());
                Some(false)
            }
            NetUiMsg::DesktopSnapshot {
                control_addr,
                jpeg_bytes,
            } => {
                self.on_net_desktop_snapshot(control_addr.clone(), jpeg_bytes.clone());
                Some(false)
            }
            NetUiMsg::DesktopFetchCycleDone => {
                self.desktop_fetch_busy = false;
                Some(false)
            }
            _ => None,
        }
    }

    fn try_net_reachability_telemetry_ops(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::HostReachability {
                control_addr,
                online,
            } => {
                self.on_net_host_reachability(control_addr.clone(), *online);
                Some(false)
            }
            NetUiMsg::ReachabilityProbeCycleDone => {
                self.reachability_probe_busy = false;
                Some(false)
            }
            NetUiMsg::HostTelemetry {
                host_key,
                gen,
                push,
            } => Some(self.on_net_host_telemetry(host_key.clone(), *gen, push.clone())),
            _ => None,
        }
    }

    fn on_net_host_ui_push_done(&mut self, ok: bool, detail: String) {
        self.host_managed_last_msg = format!("push ok={ok}: {detail}");
        self.ui_toast_text = self.host_managed_last_msg.clone();
        self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 5.0);
        self.ctx.request_repaint();
    }

    fn dispatch_net_fleet_and_error(&mut self, msg: NetUiMsg) -> bool {
        match msg {
            NetUiMsg::TelemetryLinkLost { host_key, gen } => {
                self.on_net_telemetry_link_lost(host_key, gen);
                false
            }
            NetUiMsg::FleetOpResult {
                host_key,
                ok,
                detail,
            } => {
                self.on_net_fleet_op_result(host_key, ok, detail);
                false
            }
            NetUiMsg::FleetOpDone => {
                self.fleet_busy = false;
                self.ctx.request_repaint();
                false
            }
            NetUiMsg::HostUiPushDone { ok, detail } => {
                self.on_net_host_ui_push_done(ok, detail);
                false
            }
            NetUiMsg::Error(e) => {
                self.on_net_error(e);
                false
            }
            _ => false,
        }
    }

    fn on_net_caps(&mut self, summary: String) {
        self.net_busy = false;
        self.upsert_endpoint_after_caps(summary);
        self.command_ready = true;
        self.last_net_error.clear();
        self.last_action = i18n::log_host_responded(self.ui_lang);
        self.spawn_telemetry_reader();
        self.recompute_host_connected();
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

    fn on_net_batch_stop(&mut self, succeeded: u32, failures: Vec<String>) {
        self.net_busy = false;
        self.last_net_error.clear();
        self.last_action = i18n::log_stop_vm_group(self.ui_lang, succeeded, failures.len());
        if !failures.is_empty() {
            self.last_net_error = failures.join("; ");
        }
    }

    fn on_net_batch_start(&mut self, succeeded: u32, failures: Vec<String>) {
        self.net_busy = false;
        self.last_net_error.clear();
        self.last_action = i18n::log_start_vm_group(self.ui_lang, succeeded, failures.len());
        if !failures.is_empty() {
            self.last_net_error = failures.join("; ");
        }
    }

    fn on_net_spoof_apply(&mut self, dry_run: bool, steps: Vec<String>, notes: String) {
        self.net_busy = false;
        self.last_net_error.clear();
        self.last_action = i18n::log_spoof_apply(self.ui_lang, dry_run, &steps.join(", "), &notes);
    }

    fn on_net_add_host_verify_done(
        &mut self,
        session_id: u64,
        addr: String,
        ok: bool,
        device_id: String,
        caps_summary: String,
        error: String,
    ) -> bool {
        if session_id != self.add_host_verify_session {
            return true;
        }
        self.add_host_verify_busy = false;
        self.add_host_verify_deadline = None;
        if ok {
            self.merge_add_host_after_verify(addr, device_id, caps_summary);
            self.add_host_dialog_open = false;
            self.add_host_dialog_err.clear();
            self.persist_registered_devices();
            self.last_net_error.clear();
            self.last_action = i18n::t(self.ui_lang, Msg::AddHostSavedLog).to_string();
        } else {
            tracing::debug!(%addr, %error, "add host: Hello verify failed");
            self.ui_toast_text = i18n::t(self.ui_lang, Msg::AddHostOfflineToast).to_string();
            self.ui_toast_until = Some(self.ctx.input(|i| i.time) + 3.8);
        }
        self.ctx.request_repaint();
        false
    }

    fn on_net_host_resources(
        &mut self,
        control_addr: String,
        stats: titan_common::HostResourceStats,
    ) {
        self.host_resource_stats.insert(control_addr, stats);
        self.ctx.request_repaint();
    }

    fn on_net_desktop_snapshot(&mut self, control_addr: String, jpeg_bytes: Vec<u8>) {
        match image::load_from_memory(&jpeg_bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                let tex = self.ctx.load_texture(
                    format!("host_desktop_{control_addr}"),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.host_desktop_textures.insert(control_addr, tex);
                self.ctx.request_repaint();
            }
            Err(e) => {
                tracing::warn!(
                    %control_addr,
                    %e,
                    len = jpeg_bytes.len(),
                    "desktop preview: JPEG decode failed"
                );
            }
        }
    }

    fn on_net_host_reachability(&mut self, control_addr: String, online: bool) {
        let key = Self::endpoint_addr_key(&control_addr);
        let skip_offline = !online && self.should_skip_probe_offline_for_addr(&key);
        if let Some(ep) = self
            .endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == key)
        {
            if online {
                ep.last_known_online = true;
            } else if !skip_offline {
                ep.last_known_online = false;
            }
        }
        self.ctx.request_repaint();
    }

    fn on_net_host_telemetry(
        &mut self,
        host_key: String,
        gen: u64,
        push: titan_common::ControlPush,
    ) -> bool {
        if self
            .telemetry_links
            .get(&host_key)
            .is_none_or(|l| l.session_gen != gen)
        {
            return true;
        }
        let host_key_for_ctl = host_key.clone();
        self.apply_control_push_for_telemetry(host_key, push);
        if host_key_for_ctl == Self::endpoint_addr_key(&self.control_addr) {
            self.telemetry_live = true;
            self.last_host_telemetry_at = Some(Instant::now());
        }
        self.last_net_error.clear();
        self.recompute_host_connected();
        self.ctx.request_repaint();
        false
    }

    fn on_net_telemetry_link_lost(&mut self, host_key: String, gen: u64) {
        if self
            .telemetry_links
            .get(&host_key)
            .is_none_or(|l| l.session_gen != gen)
        {
            return;
        }
        if let Some(s) = self.fleet_by_endpoint.get_mut(&host_key) {
            s.clear_telemetry();
        }
        self.host_resource_stats.remove(&host_key);
        self.host_desktop_textures.remove(&host_key);
        if host_key == Self::endpoint_addr_key(&self.control_addr) {
            self.telemetry_live = false;
            self.last_host_telemetry_at = None;
            self.recompute_host_connected();
            self.mark_control_endpoint_offline();
        }
        self.ctx.request_repaint();
    }

    fn on_net_fleet_op_result(&mut self, host_key: String, ok: bool, detail: String) {
        if !host_key.is_empty() {
            self.last_action = if ok {
                format!("{host_key}: OK")
            } else {
                format!("{host_key}: {detail}")
            };
            if !ok {
                if !self.last_net_error.is_empty() {
                    self.last_net_error.push_str("; ");
                }
                self.last_net_error
                    .push_str(&format!("{host_key}: {detail}"));
            }
        }
        self.ctx.request_repaint();
    }

    fn on_net_error(&mut self, e: String) {
        self.net_busy = false;
        self.last_net_error = e;
        self.last_action = i18n::log_request_failed(self.ui_lang);
        if !self.telemetry_live {
            self.mark_control_endpoint_offline();
        }
        self.ctx.request_repaint();
    }
}
