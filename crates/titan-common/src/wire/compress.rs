//! Optional zstd compression for control-plane sized blobs (never for raw JPEG).

/// Do not compress tiny payloads (overhead dominates).
pub const ZSTD_COMPRESS_MIN_BYTES: usize = 256;

/// zstd-compress `raw` at a moderate level (tuned for LAN control messages).
pub fn zstd_compress_all(raw: &[u8]) -> std::io::Result<Vec<u8>> {
    zstd::encode_all(raw, 3)
}

pub fn zstd_decompress_all(buf: &[u8]) -> std::io::Result<Vec<u8>> {
    zstd::decode_all(buf)
}

/// Compress with zstd only when large enough and strictly smaller than input.
pub fn maybe_zstd_compress(raw: &[u8]) -> std::io::Result<std::borrow::Cow<'_, [u8]>> {
    if raw.len() < ZSTD_COMPRESS_MIN_BYTES {
        return Ok(std::borrow::Cow::Borrowed(raw));
    }
    let compressed = zstd_compress_all(raw)?;
    if compressed.len() >= raw.len() {
        return Ok(std::borrow::Cow::Borrowed(raw));
    }
    Ok(std::borrow::Cow::Owned(compressed))
}
