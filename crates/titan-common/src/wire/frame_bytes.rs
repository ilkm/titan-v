//! `bytes::BytesMut` helpers for length-prefixed reads (pairs with existing `FRAME_HEADER_LEN`).

use bytes::{Buf, Bytes, BytesMut};

/// After the caller has verified `buf` holds at least `payload_len` bytes, split them off as `Bytes`.
pub fn take_payload_bytes(buf: &mut BytesMut, payload_len: usize) -> Option<Bytes> {
    if buf.len() < payload_len {
        return None;
    }
    Some(buf.split_to(payload_len).freeze())
}

/// Advance `buf` by `n` without allocating (discard).
pub fn skip_bytes(buf: &mut BytesMut, n: usize) -> bool {
    if buf.len() < n {
        return false;
    }
    buf.advance(n);
    true
}
