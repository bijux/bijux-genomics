//! Owner: runner bundle filesystem paths, persistence support, and content identities.
use anyhow::{anyhow, Context, Result};
pub use bijux_dna_bench_model::sidecar_path_for;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
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
/// Keep private plaintext owned through backend execution and report cleanup errors.
pub(super) fn with_staged_plaintext<T>(
    output_path: &Path,
    plaintext: &[u8],
    operation: impl FnOnce(&Path) -> Result<T>,
) -> Result<T> {
    use std::io::Write;
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut staged = bijux_dna_infra::private_temp_file_in(parent, ".bundle-plaintext-")
        .context("create private bundle plaintext")?;
    let result = staged
        .as_file_mut()
        .write_all(plaintext)
        .context("write private bundle plaintext")
        .and_then(|()| operation(staged.path()));
    let cleanup = staged.close().context("remove private bundle plaintext");
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(anyhow!("{error:#}; cleanup also failed: {cleanup:#}")),
    }
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
