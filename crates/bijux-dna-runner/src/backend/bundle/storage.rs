//! Owner: runner bundle filesystem paths, persistence support, and content identities.
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
#[must_use]
pub fn sidecar_path_for(bundle_path: &Path) -> PathBuf {
    let mut path = bundle_path.to_path_buf();
    let suffix = bundle_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle.sidecar.json".to_string(), |name| format!("{name}.sidecar.json"));
    path.set_file_name(suffix);
    path
}
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}
/// Hash a bundle without interpreting its content.
///
/// # Errors
/// Returns an error when the file cannot be opened or read.
pub fn digest_file_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let count = file.read(&mut buf).with_context(|| format!("read {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buf[..count]);
    }
    Ok(hex_encode(&hasher.finalize()))
}
pub(super) fn recipient_fingerprint(value: &str) -> String {
    let digest = sha256_hex(value.as_bytes());
    digest.chars().take(16).collect()
}
pub(super) fn ensure_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow!("path has no parent: {}", path.display()));
    };
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))
}
pub(super) fn temp_output_path(output_path: &Path) -> PathBuf {
    let mut path = output_path.to_path_buf();
    let suffix = output_path.file_name().and_then(|name| name.to_str()).map_or_else(
        || "bundle.tmp".to_string(),
        |name| format!("{name}.tmp.{}.{}", std::process::id(), monotonic_nanos()),
    );
    path.set_file_name(suffix);
    path
}
pub(super) fn temp_plaintext_path(output_path: &Path) -> PathBuf {
    let mut path = temp_output_path(output_path);
    let suffix = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle.input".to_string(), |name| format!("{name}.input"));
    path.set_file_name(suffix);
    path
}
pub(super) fn monotonic_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_nanos())
}
pub(super) fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
