use std::collections::BTreeMap;
use std::fmt::Write as _;

use anyhow::Result;
use sha2::Digest;

/// Compute a stable invocation hash for a runner invocation.
///
/// Inputs include argv, env, image digest, and input hashes.
/// # Errors
/// Returns an error if canonical serialization fails.
pub fn invocation_hash(
    argv: &[String],
    env: &BTreeMap<String, String>,
    image_digest: &str,
    input_hashes: &[String],
) -> Result<String> {
    let mut inputs = input_hashes.to_vec();
    inputs.sort();
    let payload = serde_json::json!({
        "argv": argv,
        "env": env,
        "image_digest": image_digest,
        "inputs": inputs,
    });
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(sha256_hex(hasher.finalize()))
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
