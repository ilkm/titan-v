//! rkyv-backed fleet ping payload (shared layout helpers with the main TCP control plane).

use bytes::Bytes;
use rkyv::{Archive, Deserialize, Serialize};

/// Minimal rkyv message for QUIC/TCP v2 handshakes and tests.
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FleetRkyvPing {
    pub layout_version: u32,
    pub nonce: u64,
}

/// Serialize `FleetRkyvPing` to owned bytes (aligned serialization buffer).
pub fn fleet_rkyv_encode_ping(p: &FleetRkyvPing) -> Result<Bytes, String> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(p).map_err(|e| e.to_string())?;
    Ok(Bytes::copy_from_slice(aligned.as_slice()))
}

/// Deserialize `FleetRkyvPing` from a byte slice.
pub fn fleet_rkyv_decode_ping(buf: &[u8]) -> Result<FleetRkyvPing, String> {
    rkyv::from_bytes::<FleetRkyvPing, rkyv::rancor::Error>(buf).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_rkyv_ping_roundtrip() {
        let p = FleetRkyvPing {
            layout_version: 1,
            nonce: 0xdead_beef,
        };
        let b = fleet_rkyv_encode_ping(&p).unwrap();
        let q = fleet_rkyv_decode_ping(&b).unwrap();
        assert_eq!(p, q);
    }
}
