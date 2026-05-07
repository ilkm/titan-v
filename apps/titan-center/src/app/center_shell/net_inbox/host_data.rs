use crate::app::CenterApp;
use crate::app::net::NetUiMsg;

impl CenterApp {
    fn try_net_desktop_frame_payload(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::DesktopSnapshot {
                control_addr,
                jpeg_bytes,
            } => {
                self.on_net_desktop_snapshot(control_addr.clone(), jpeg_bytes.clone());
                Some(false)
            }
            NetUiMsg::DesktopFrameDecoded {
                control_addr,
                width,
                height,
                rgba_bytes,
            } => {
                self.on_net_desktop_frame_decoded(
                    control_addr.clone(),
                    *width,
                    *height,
                    rgba_bytes.clone(),
                );
                Some(false)
            }
            NetUiMsg::DesktopFetchCycleDone => {
                self.desktop_fetch_busy = false;
                Some(false)
            }
            _ => None,
        }
    }

    pub(super) fn try_net_host_announced_only(&mut self, msg: &NetUiMsg) -> Option<bool> {
        match msg {
            NetUiMsg::HostAnnounced {
                quic_addr,
                label,
                source_ip,
                device_id,
                fingerprint,
            } => {
                self.apply_net_host_announced(
                    quic_addr.clone(),
                    label.clone(),
                    source_ip.clone(),
                    device_id.clone(),
                    fingerprint.clone(),
                );
                Some(false)
            }
            _ => None,
        }
    }

    pub(super) fn try_net_host_resources_payload(&mut self, msg: &NetUiMsg) -> Option<bool> {
        if let Some(done) = self.try_net_desktop_frame_payload(msg) {
            return Some(done);
        }
        match msg {
            NetUiMsg::HostResources {
                control_addr,
                stats,
            } => {
                self.on_net_host_resources(control_addr.clone(), stats.clone());
                Some(false)
            }
            _ => None,
        }
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
        // Backward-compatible fallback: decode in background if old message still appears.
        let tx = self.net_tx.clone();
        let _ = std::thread::Builder::new()
            .name("titan-center-desktop-decode".into())
            .spawn(move || match image::load_from_memory(&jpeg_bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let msg = NetUiMsg::DesktopFrameDecoded {
                        control_addr,
                        width: rgba.width() as usize,
                        height: rgba.height() as usize,
                        rgba_bytes: rgba.into_vec(),
                    };
                    let _ = tx.send(msg);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        len = jpeg_bytes.len(),
                        "desktop preview: JPEG decode failed"
                    );
                }
            });
    }

    fn on_net_desktop_frame_decoded(
        &mut self,
        control_addr: String,
        width: usize,
        height: usize,
        rgba_bytes: Vec<u8>,
    ) {
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([width, height], rgba_bytes.as_slice());
        let tex = self.ctx.load_texture(
            format!("host_desktop_{control_addr}"),
            color_image,
            egui::TextureOptions::LINEAR,
        );
        self.host_desktop_textures.insert(control_addr, tex);
        self.ctx.request_repaint();
    }

    pub(super) fn on_net_host_reachability(&mut self, control_addr: String, online: bool) {
        let key = Self::endpoint_addr_key(&control_addr);
        let skip_offline = !online
            && (self.should_skip_probe_offline_for_addr(&key)
                || self.has_running_telemetry_link_for_addr(&key));
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
}
