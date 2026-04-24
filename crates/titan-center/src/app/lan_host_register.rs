//! UDP listener: `titan-host serve` announces its control-plane TCP address so the center can add it to the device list.

use std::net::UdpSocket;
use std::sync::mpsc::Sender;

use egui::Context;
use titan_common::HostAnnounceBeacon;

use super::net_msg::NetUiMsg;

pub fn spawn_center_lan_host_register_listener(
    tx: Sender<NetUiMsg>,
    ctx: Context,
    listen_port: u16,
) {
    std::thread::spawn(move || center_lan_register_loop(tx, ctx, listen_port));
}

fn center_lan_register_loop(tx: Sender<NetUiMsg>, ctx: Context, listen_port: u16) {
    let port = listen_port.max(1);
    let sock = match UdpSocket::bind(("0.0.0.0", port)) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                error = %e,
                port,
                "LAN host register: UDP bind failed (port may be in use)"
            );
            return;
        }
    };
    tracing::info!(port, "LAN host register: listening for host announcements");
    let mut buf = vec![0u8; 4096];
    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, _from)) => {
                let slice = &buf[..n];
                let beacon: HostAnnounceBeacon = match serde_json::from_slice(slice) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                if beacon.validate().is_err() {
                    continue;
                }
                let addr = beacon.host_control_addr.trim().to_string();
                if addr.is_empty() {
                    continue;
                }
                let label_raw = beacon.label.trim();
                let label = if label_raw.is_empty() {
                    format!("host-{}", addr.replace([':', '.'], "-"))
                } else {
                    label_raw.to_string()
                };
                let device_id = beacon.device_id.trim().to_string();
                if tx
                    .send(NetUiMsg::HostAnnounced {
                        control_addr: addr,
                        label,
                        device_id,
                    })
                    .is_err()
                {
                    break;
                }
                ctx.request_repaint();
            }
            Err(e) => {
                tracing::debug!(error = %e, "LAN host register recv");
            }
        }
    }
}
