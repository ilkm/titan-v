//! LAN host announce merge for [`NetUiMsg::HostAnnounced`](crate::app::net::NetUiMsg).

use crate::app::CenterApp;
use crate::app::i18n;
use crate::app::persist_data::HostEndpoint;
use if_addrs::IfAddr;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use titan_common::ipv4_in_subnet;

impl CenterApp {
    pub(crate) fn apply_net_host_announced(
        &mut self,
        quic_addr: String,
        label: String,
        source_ip: String,
        device_id: String,
        fingerprint: String,
    ) {
        let addr = Self::endpoint_addr_key(&quic_addr);
        if addr.is_empty() {
            return;
        }
        if !self.allow_announced_source_and_addr(&source_ip, &quic_addr) {
            return;
        }
        let id_from_host = device_id.trim().to_string();
        let resolved_label = resolve_announced_label(&label, &addr);
        self.dispatch_announced_merge(&addr, &quic_addr, &resolved_label, &id_from_host);
        self.auto_trust_announced_host(&fingerprint, &id_from_host, &resolved_label);
        self.finish_net_host_announced_merge(&resolved_label, &addr);
        self.spawn_ui_lang_push_to_host_control_addr(&quic_addr);
        self.push_initial_vm_window_snapshot_to(&quic_addr, &id_from_host);
        self.maybe_event_reconnect_on_announce(&addr);
    }

    /// If the announce came from the currently-selected control host and we are not connected,
    /// drop any cached (possibly stale) QUIC connection and arm an immediate `auto_hello`.
    /// Combined with the host-side initial burst (50 ms × 10), this gets a fresh hello in flight
    /// within a single LAN RTT of the host coming back up.
    fn maybe_event_reconnect_on_announce(&mut self, announced_key: &str) {
        let control_key = Self::endpoint_addr_key(&self.control_addr);
        if self.is_control_connected() {
            return;
        }
        if announced_key != control_key {
            return;
        }
        self.force_reconnect_to_control_host();
        self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
        self.ctx.request_repaint();
    }

    fn auto_trust_announced_host(&self, fingerprint: &str, device_id: &str, label: &str) {
        if fingerprint.len() != 64 {
            return;
        }
        if let Err(e) = self
            .center_security
            .trust
            .upsert(build_lan_trust_entry(fingerprint, label))
        {
            tracing::warn!(error = %e, %device_id, "auto-trust upsert failed");
        }
    }

    fn push_initial_vm_window_snapshot_to(&self, control_addr: &str, device_id: &str) {
        if device_id.is_empty() {
            return;
        }
        let Some(ep) = self
            .endpoints
            .iter()
            .find(|e| Self::endpoint_addr_key(&e.addr) == Self::endpoint_addr_key(control_addr))
            .cloned()
        else {
            return;
        };
        crate::app::vm_window_push_to_hosts::push_snapshot_to_endpoint(
            &ep,
            &self.vm_window_records,
        );
    }

    fn dispatch_announced_merge(
        &mut self,
        addr: &str,
        control_addr: &str,
        resolved_label: &str,
        id_from_host: &str,
    ) {
        let new_addr = control_addr.trim().to_string();
        let new_key = Self::endpoint_addr_key(&new_addr);
        let lone_legacy = self.lone_legacy_endpoint_index();
        if !id_from_host.is_empty() {
            self.merge_announced_nonempty_device_id(
                addr,
                id_from_host,
                &new_addr,
                &new_key,
                resolved_label,
                lone_legacy,
            );
        } else {
            self.merge_announced_empty_device_id(addr, resolved_label);
        }
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
            self.sync_vm_windows_for_device_rebind(id_from_host, new_addr, resolved_label);
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
        self.sync_vm_windows_for_device_rebind(id_from_host, new_addr, resolved_label);
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
                let control_addr = self.control_addr.clone();
                self.mark_command_ready_for_addr(&control_addr, false);
                self.auto_hello_accum = Self::AUTO_HELLO_RETRY_SECS;
            }
        }
        let ep = &mut self.endpoints[pos];
        ep.addr = new_addr.to_string();
        if ep.label != resolved_label {
            ep.label = resolved_label.to_string();
        }
    }

    fn allow_announced_source_and_addr(&self, source_ip: &str, quic_addr: &str) -> bool {
        if self.discovery_bind_ipv4s.is_empty() {
            return true;
        }
        let Some(source_v4) = parse_ipv4(source_ip) else {
            tracing::warn!(%source_ip, "lan announce rejected: source IP is not IPv4");
            return false;
        };
        let selected = selected_bind_networks(&self.discovery_bind_ipv4s);
        if selected.is_empty() {
            tracing::warn!(
                "lan announce rejected: no selected bind interfaces are currently active"
            );
            return false;
        }
        if !is_ipv4_in_any_selected_subnet(source_v4, &selected) {
            tracing::debug!(%source_ip, "lan announce rejected: source not in selected bind subnets");
            return false;
        }
        if !is_announced_addr_allowed(quic_addr, source_ip, &selected) {
            return false;
        }
        true
    }

    fn sync_vm_windows_for_device_rebind(
        &mut self,
        device_id: &str,
        new_addr: &str,
        resolved_label: &str,
    ) {
        let db_path = crate::app::vm_window_db::center_vm_window_db_path();
        for row in self
            .vm_window_records
            .iter_mut()
            .filter(|r| r.device_id.trim() == device_id)
        {
            let mut row_changed = false;
            if row.host_control_addr != new_addr {
                row.host_control_addr = new_addr.to_string();
                row_changed = true;
            }
            if row.host_label != resolved_label {
                row.host_label = resolved_label.to_string();
                row_changed = true;
            }
            if !row_changed {
                continue;
            }
            if let Err(e) = crate::app::vm_window_db::upsert(&db_path, row) {
                tracing::warn!(error = %e, record_id = %row.record_id, "vm_window_db upsert failed after host rebind");
            }
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

fn build_lan_trust_entry(fingerprint: &str, label: &str) -> titan_quic::TrustEntry {
    titan_quic::TrustEntry {
        fingerprint: fingerprint.to_string(),
        label: label.to_string(),
        role: "host".to_string(),
        source: "lan-announce".to_string(),
        added_at_epoch_s: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default(),
    }
}

fn parse_ipv4(s: &str) -> Option<Ipv4Addr> {
    s.parse::<IpAddr>().ok().and_then(ipv4_from_ip_addr)
}

fn parse_quic_addr_ipv4(s: &str) -> Option<Ipv4Addr> {
    s.parse::<SocketAddr>()
        .ok()
        .and_then(|addr| ipv4_from_ip_addr(addr.ip()))
}

fn ipv4_from_ip_addr(ip: IpAddr) -> Option<Ipv4Addr> {
    match ip {
        IpAddr::V4(v4) => Some(v4),
        IpAddr::V6(_) => None,
    }
}

fn selected_bind_networks(selected_bind_ips: &[String]) -> Vec<(Ipv4Addr, Ipv4Addr)> {
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return Vec::new();
    };
    let selected: HashSet<Ipv4Addr> = selected_bind_ips
        .iter()
        .filter_map(|s| s.parse::<Ipv4Addr>().ok())
        .collect();
    let mut nets = Vec::new();
    for iface in ifaces {
        let IfAddr::V4(v4) = iface.addr else {
            continue;
        };
        if !selected.contains(&v4.ip) {
            continue;
        }
        nets.push((v4.ip, v4.netmask));
    }
    nets
}

fn is_ipv4_in_any_selected_subnet(target: Ipv4Addr, nets: &[(Ipv4Addr, Ipv4Addr)]) -> bool {
    nets.iter()
        .any(|(ip, mask)| ipv4_in_subnet(target, *ip, *mask))
}

fn is_announced_addr_allowed(
    quic_addr: &str,
    source_ip: &str,
    selected: &[(Ipv4Addr, Ipv4Addr)],
) -> bool {
    if let Some(announced) = parse_quic_addr_ipv4(quic_addr)
        && !is_ipv4_in_any_selected_subnet(announced, selected)
    {
        tracing::debug!(
            %quic_addr,
            %source_ip,
            "lan announce rejected: announced addr not in selected bind subnets"
        );
        return false;
    }
    true
}
