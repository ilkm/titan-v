//! Telemetry push decode / texture upload for the fleet TCP reader.

use std::time::Instant;

use crate::app::CenterApp;

impl CenterApp {
    pub(crate) fn apply_control_push_for_telemetry(
        &mut self,
        host_key: String,
        push: titan_common::ControlPush,
    ) {
        match push {
            titan_common::ControlPush::HostTelemetry {
                vms,
                volumes,
                content_hint,
            } => self.apply_cp_host_telemetry(host_key, vms, volumes, content_hint),
            titan_common::ControlPush::HostResourceLive { stats } => {
                self.apply_cp_host_resource_live(host_key, stats);
            }
            titan_common::ControlPush::HostDesktopPreviewJpeg { jpeg_bytes, .. } => {
                self.apply_cp_desktop_jpeg(host_key, jpeg_bytes);
            }
            // Liveness-only pushes carry no display payload; presence alone refreshes
            // `last_host_telemetry_at` in `on_net_host_telemetry`.
            titan_common::ControlPush::HostHeartbeat { .. } => {}
            // Tear-down is handled at the higher level in `on_net_host_telemetry`.
            titan_common::ControlPush::HostByeNow => {}
        }
    }

    fn apply_cp_host_telemetry(
        &mut self,
        host_key: String,
        vms: Vec<titan_common::VmBrief>,
        volumes: Vec<titan_common::DiskVolume>,
        content_hint: Option<String>,
    ) {
        let st = self.fleet_by_endpoint.entry(host_key.clone()).or_default();
        let n = vms.len();
        st.vms = vms.clone();
        st.volumes = volumes.clone();
        st.telemetry_live = true;
        st.last_telemetry_at = Some(Instant::now());
        self.apply_cp_telemetry_selected_host(&host_key, &vms, &volumes, n);
        self.apply_cp_telemetry_endpoint_rows(&host_key, n);
        if let Some(h) = content_hint
            && !h.is_empty()
        {
            self.last_action = h;
        }
    }

    fn apply_cp_telemetry_selected_host(
        &mut self,
        host_key: &str,
        vms: &[titan_common::VmBrief],
        volumes: &[titan_common::DiskVolume],
        n: usize,
    ) {
        if self.selected_endpoint_key().as_deref() != Some(host_key) {
            return;
        }
        self.vm_inventory = vms.to_vec();
        self.host_disk_volumes = volumes.to_vec();
        if let Some(ep) = self.endpoint_mut_for_control_addr() {
            ep.last_vm_count = n as u32;
            ep.last_known_online = true;
        }
    }

    fn apply_cp_telemetry_endpoint_rows(&mut self, host_key: &str, n: usize) {
        if let Some(ep) = self
            .endpoints
            .iter_mut()
            .find(|e| Self::endpoint_addr_key(&e.addr) == host_key)
        {
            ep.last_vm_count = n as u32;
            ep.last_known_online = true;
        }
    }

    fn apply_cp_host_resource_live(
        &mut self,
        host_key: String,
        stats: titan_common::HostResourceStats,
    ) {
        if !host_key.is_empty() {
            self.host_resource_stats.insert(host_key, stats);
        }
    }

    fn apply_cp_desktop_jpeg(&mut self, host_key: String, jpeg_bytes: Vec<u8>) {
        if host_key.is_empty() {
            return;
        }
        match image::load_from_memory(&jpeg_bytes) {
            Ok(img) => self.insert_desktop_texture_from_rgba(host_key, img),
            Err(e) => tracing::warn!(
                %host_key,
                %e,
                len = jpeg_bytes.len(),
                "telemetry desktop preview: JPEG decode failed"
            ),
        }
    }

    fn insert_desktop_texture_from_rgba(&mut self, host_key: String, img: image::DynamicImage) {
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let tex = self.ctx.load_texture(
            format!("host_desktop_{host_key}"),
            color_image,
            egui::TextureOptions::LINEAR,
        );
        self.host_desktop_textures.insert(host_key, tex);
    }
}
