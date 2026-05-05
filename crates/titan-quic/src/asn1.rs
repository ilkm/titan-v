//! Minimal DER walker for extracting the **SPKI** subtree of a self-signed X.509 cert.
//!
//! Why bother: rcgen-issued certs are self-signed leaves with the layout
//! ```text
//! Certificate(SEQ){
//!   tbsCertificate(SEQ){
//!     [0] EXPLICIT version (optional),
//!     serial INTEGER, signature SEQ, issuer SEQ, validity SEQ, subject SEQ,
//!     subjectPublicKeyInfo SEQ,
//!     ...
//!   },
//!   sigAlgorithm SEQ, signature BITSTRING
//! }
//! ```
//! We do not want a full X.509 dep just to get the SPKI bytes; the layout is fixed and the
//! cert never crosses the wire to an attacker who could mess with it (it is the peer's own
//! cert, validated by rustls signatures separately).

use anyhow::{Result, anyhow};

/// Returns the **TLV** of the `subjectPublicKeyInfo` field within a self-signed X.509 cert.
pub fn extract_spki_der(cert_der: &[u8]) -> Result<Vec<u8>> {
    let mut top = Asn1Walker::new(cert_der);
    let mut cert_seq = top.into_inner_sequence()?;
    let mut tbs = cert_seq.into_inner_sequence()?;
    if tbs.peek_tag()? == 0xA0 {
        tbs.skip_one()?;
    }
    tbs.skip_one()?;
    tbs.skip_one()?;
    tbs.skip_one()?;
    tbs.skip_one()?;
    tbs.skip_one()?;
    tbs.copy_one_tlv()
}

pub struct Asn1Walker<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Asn1Walker<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn peek_tag(&self) -> Result<u8> {
        self.buf
            .get(self.pos)
            .copied()
            .ok_or_else(|| anyhow!("asn1: eof at tag"))
    }

    fn read_tlv(&mut self) -> Result<(u8, usize, usize)> {
        let tag = self.peek_tag()?;
        self.pos += 1;
        let first = *self
            .buf
            .get(self.pos)
            .ok_or_else(|| anyhow!("asn1: eof at len"))?;
        self.pos += 1;
        let len = self.read_len(first)?;
        if self.pos + len > self.buf.len() {
            return Err(anyhow!("asn1: tlv out of range"));
        }
        Ok((tag, self.pos, len))
    }

    fn read_len(&mut self, first: u8) -> Result<usize> {
        if first & 0x80 == 0 {
            return Ok(first as usize);
        }
        let n = (first & 0x7F) as usize;
        if n == 0 || n > 8 {
            return Err(anyhow!("asn1: bad long-form length"));
        }
        let mut v = 0usize;
        for _ in 0..n {
            let b = *self
                .buf
                .get(self.pos)
                .ok_or_else(|| anyhow!("asn1: eof in long len"))?;
            v = (v << 8) | b as usize;
            self.pos += 1;
        }
        Ok(v)
    }

    pub fn into_inner_sequence(&mut self) -> Result<Asn1Walker<'a>> {
        let (tag, value_start, len) = self.read_tlv()?;
        if tag != 0x30 {
            return Err(anyhow!("asn1: expected SEQUENCE got {tag:#x}"));
        }
        self.pos = value_start + len;
        Ok(Asn1Walker {
            buf: &self.buf[value_start..value_start + len],
            pos: 0,
        })
    }

    pub fn skip_one(&mut self) -> Result<()> {
        let (_, value_start, len) = self.read_tlv()?;
        self.pos = value_start + len;
        Ok(())
    }

    pub fn copy_one_tlv(&mut self) -> Result<Vec<u8>> {
        let start = self.pos;
        self.skip_one()?;
        Ok(self.buf[start..self.pos].to_vec())
    }
}
