//! Self-signed mTLS identity (cert + key) for one Titan-v node.
//!
//! Behaviour:
//! * On first use, [`load_or_generate`] creates a fresh ed25519 self-signed cert with
//!   `CN/SAN.DNS = titan-{role}-{device_id}` and writes both PEM files atomically.
//! * Subsequent calls load the existing files; the SPKI fingerprint is therefore stable
//!   across restarts and is what the trust store keys on (CA-less mTLS).
//! * Permissions on the key file are tightened to `0o600` on Unix; on Windows the parent
//!   directory inherits the user profile ACL.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use sha2::{Digest, Sha256};

const CERT_FILE: &str = "identity.cert.pem";
const KEY_FILE: &str = "identity.key.pem";
const CERT_VALIDITY_DAYS: i64 = 365 * 100;

/// Which side of the bus this identity belongs to (changes only the SAN/CN label).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Center,
    Host,
}

impl Role {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Center => "center",
            Role::Host => "host",
        }
    }
}

/// PEM-encoded cert + key plus the cached SPKI fingerprint (lowercase hex, 64 chars).
#[derive(Debug, Clone)]
pub struct Identity {
    pub role: Role,
    pub device_id: String,
    pub cert_pem: String,
    pub key_pem: String,
    pub spki_sha256_hex: String,
    pub cert_der: Vec<u8>,
    pub key_pkcs8_der: Vec<u8>,
}

/// Loads `dir/identity.{cert,key}.pem`; generates them if missing.
pub fn load_or_generate(dir: &Path, role: Role, device_id: &str) -> Result<Identity> {
    fs::create_dir_all(dir).with_context(|| format!("create identity dir {}", dir.display()))?;
    let cert_path = dir.join(CERT_FILE);
    let key_path = dir.join(KEY_FILE);
    if cert_path.exists() && key_path.exists() {
        return load_existing(&cert_path, &key_path, role, device_id);
    }
    let fresh = generate_new(role, device_id)?;
    write_atomic(&cert_path, fresh.cert_pem.as_bytes())?;
    write_key_atomic(&key_path, fresh.key_pem.as_bytes())?;
    Ok(fresh)
}

fn load_existing(
    cert_path: &Path,
    key_path: &Path,
    role: Role,
    device_id: &str,
) -> Result<Identity> {
    let cert_pem = fs::read_to_string(cert_path)
        .with_context(|| format!("read cert {}", cert_path.display()))?;
    let key_pem =
        fs::read_to_string(key_path).with_context(|| format!("read key {}", key_path.display()))?;
    let cert_der = pem_to_der(&cert_pem, "CERTIFICATE")?;
    let key_pkcs8_der = pem_to_pkcs8_der(&key_pem)?;
    let spki = x509_extract_spki_der(&cert_der)?;
    let spki_sha256_hex = sha256_hex(&spki);
    Ok(Identity {
        role,
        device_id: device_id.to_string(),
        cert_pem,
        key_pem,
        spki_sha256_hex,
        cert_der,
        key_pkcs8_der,
    })
}

fn generate_new(role: Role, device_id: &str) -> Result<Identity> {
    let cn = format!("titan-{}-{}", role.as_str(), device_id);
    let key_pair = KeyPair::generate_for(&rcgen::PKCS_ED25519).context("rcgen ed25519 keypair")?;
    let mut params = CertificateParams::new(vec![cn.clone()]).context("rcgen cert params")?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, cn.clone());
    params.distinguished_name = dn;
    params.subject_alt_names = vec![SanType::DnsName(
        cn.clone().try_into().context("rcgen SAN dns conversion")?,
    )];
    params.not_before = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
    params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(CERT_VALIDITY_DAYS);
    let cert = params.self_signed(&key_pair).context("rcgen self_signed")?;
    let cert_pem = cert.pem();
    let cert_der = cert.der().to_vec();
    let key_pem = key_pair.serialize_pem();
    let key_pkcs8_der = key_pair.serialize_der();
    let spki = x509_extract_spki_der(&cert_der)?;
    let spki_sha256_hex = sha256_hex(&spki);
    Ok(Identity {
        role,
        device_id: device_id.to_string(),
        cert_pem,
        key_pem,
        spki_sha256_hex,
        cert_der,
        key_pkcs8_der,
    })
}

fn write_atomic(target: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = tmp_path(target);
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, target).with_context(|| format!("rename to {}", target.display()))?;
    Ok(())
}

fn write_key_atomic(target: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = tmp_path(target);
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    set_user_only_perms(&tmp);
    fs::rename(&tmp, target).with_context(|| format!("rename to {}", target.display()))?;
    set_user_only_perms(target);
    Ok(())
}

fn tmp_path(target: &Path) -> PathBuf {
    let mut tmp = target.to_path_buf();
    let mut name = target
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    tmp.set_file_name(name);
    tmp
}

#[cfg(unix)]
fn set_user_only_perms(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(p) {
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        let _ = fs::set_permissions(p, perms);
    }
}

#[cfg(not(unix))]
fn set_user_only_perms(_p: &Path) {}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn pem_to_der(pem: &str, label: &str) -> Result<Vec<u8>> {
    let begin = format!("-----BEGIN {label}-----");
    let end = format!("-----END {label}-----");
    let start = pem
        .find(&begin)
        .with_context(|| format!("PEM missing {label}"))?;
    let body_start = start + begin.len();
    let body_end_rel = pem[body_start..]
        .find(&end)
        .with_context(|| format!("PEM unterminated {label}"))?;
    let body = &pem[body_start..body_start + body_end_rel];
    let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    base64_decode(&cleaned).with_context(|| format!("base64 decode {label}"))
}

fn pem_to_pkcs8_der(pem: &str) -> Result<Vec<u8>> {
    pem_to_der(pem, "PRIVATE KEY")
        .or_else(|_| pem_to_der(pem, "EC PRIVATE KEY"))
        .or_else(|_| pem_to_der(pem, "RSA PRIVATE KEY"))
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64ct::{Base64, Encoding};
    Ok(Base64::decode_vec(s)?)
}

fn x509_extract_spki_der(cert_der: &[u8]) -> Result<Vec<u8>> {
    crate::asn1::extract_spki_der(cert_der)
}
