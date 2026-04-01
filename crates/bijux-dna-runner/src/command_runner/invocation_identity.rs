use std::collections::BTreeMap;

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
    Ok(format!("{:x}", hasher.finalize()))
}
