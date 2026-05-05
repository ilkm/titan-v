//! UDP listener: `titan-host serve` announces its control-plane TCP address so the center can add it to the device list.

use std::net::UdpSocket;
use std::sync::mpsc::Sender;

use egui::Context;
use titan_common::HostAnnounceBeacon;

use super::net::NetUiMsg;

pub fn spawn_center_lan_host_register_listener(
    tx: Sender<NetUiMsg>,
    ctx: Context,
    listen_port: u16,
) {
    std::thread::spawn(move || center_lan_register_loop(tx, ctx, listen_port));
}

fn lan_register_bind(port: u16) -> Option<UdpSocket> {
    match UdpSocket::bind(("0.0.0.0", port)) {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!(
                error = %e,
                port,
                "LAN host register: UDP bind failed (port may be in use)"
            );
            None
        }
    }
}

fn lan_register_label_for_beacon(addr: &str, label_raw: &str) -> String {
    if label_raw.is_empty() {
        format!("host-{}", addr.replace([':', '.'], "-"))
    } else {
        label_raw.to_string()
    }
}

fn lan_register_parse_announced(slice: &[u8]) -> Option<(String, String, String, String)> {
    let beacon: HostAnnounceBeacon = serde_json::from_slice(slice).ok()?;
    beacon.validate().ok()?;
    let addr = beacon.host_quic_addr.trim().to_string();
    if addr.is_empty() {
        return None;
    }
    let label = lan_register_label_for_beacon(&addr, beacon.label.trim());
    let device_id = beacon.device_id.trim().to_string();
    let fingerprint = beacon.host_spki_sha256_hex.trim().to_string();
    Some((addr, label, device_id, fingerprint))
}

fn lan_register_try_dispatch(slice: &[u8], tx: &Sender<NetUiMsg>, ctx: &Context) -> bool {
    let Some((addr, label, device_id, fingerprint)) = lan_register_parse_announced(slice) else {
        return true;
    };
    if tx
        .send(NetUiMsg::HostAnnounced {
            quic_addr: addr,
            label,
            device_id,
            fingerprint,
        })
        .is_err()
    {
        return false;
    }
    ctx.request_repaint();
    true
}

fn center_lan_register_loop(tx: Sender<NetUiMsg>, ctx: Context, listen_port: u16) {
    let port = listen_port.max(1);
    let Some(sock) = lan_register_bind(port) else {
        return;
    };
    tracing::info!(port, "LAN host register: listening for host announcements");
    let mut buf = vec![0u8; 4096];
    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, _from)) => {
                if !lan_register_try_dispatch(&buf[..n], &tx, &ctx) {
                    break;
                }
            }
            Err(e) => {
                tracing::debug!(error = %e, "LAN host register recv");
            }
        }
    }
}
