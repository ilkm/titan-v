use titan_common::ControlResponse;

use super::super::errors::ServeError;

pub(super) async fn handle_host_resource_snapshot() -> Result<ControlResponse, ServeError> {
    let join = tokio::task::spawn_blocking(crate::host_resources::collect_blocking)
        .await
        .map_err(|e| super::join_io(format!("join: {e}")))?;
    Ok(ControlResponse::HostResourceSnapshot { stats: join })
}

pub(super) async fn handle_host_desktop_snapshot(
    max_width: u32,
    max_height: u32,
    jpeg_quality: u8,
) -> Result<ControlResponse, ServeError> {
    let mw = max_width.clamp(320, 4096);
    let mh = max_height.clamp(240, 4096);
    let q = jpeg_quality.clamp(1, 95);
    let join = tokio::task::spawn_blocking(move || {
        crate::desktop_snapshot::capture_primary_display_jpeg(mw, mh, q)
    })
    .await
    .map_err(|e| super::join_io(format!("join: {e}")))?;
    match join {
        Ok((jpeg_bytes, width_px, height_px)) => {
            desktop_jpeg_response_or_limit(jpeg_bytes, width_px, height_px)
        }
        Err(e) => Ok(super::server_err(500, e)),
    }
}

fn desktop_jpeg_response_or_limit(
    jpeg_bytes: Vec<u8>,
    width_px: u32,
    height_px: u32,
) -> Result<ControlResponse, ServeError> {
    let max = titan_common::MAX_PAYLOAD_BYTES as usize;
    if jpeg_bytes.len() > max.saturating_sub(512) {
        return Ok(super::server_err(
            413,
            format!(
                "desktop JPEG {} bytes exceeds wire limit (~{} bytes); lower resolution or quality",
                jpeg_bytes.len(),
                max
            ),
        ));
    }
    Ok(ControlResponse::DesktopSnapshotJpeg {
        jpeg_bytes,
        width_px,
        height_px,
    })
}
