use sha2::Digest;

use crate::foundation::{canonical::parameters_json_canonicalization, Result};

/// Canonical params hash for run identity.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(&parameters_json_canonicalization(params))?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Deterministic parameters fingerprint for cache keys.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn parameters_fingerprint(params: &serde_json::Value) -> Result<String> {
    params_hash(params)
}

/// Deterministic run id derived from pipeline identity and hashes.
#[must_use]
pub fn run_id_from_hashes(
    pipeline_id: &str,
    sample_id: &str,
    params_hash: &str,
    input_hashes: &[String],
    reference_genome: Option<&str>,
) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(pipeline_id.as_bytes());
    hasher.update(b"|");
    hasher.update(sample_id.as_bytes());
    hasher.update(b"|");
    hasher.update(params_hash.as_bytes());
    hasher.update(b"|");
    for hash in input_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|");
    if let Some(reference) = reference_genome {
        hasher.update(reference.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[must_use]
pub fn input_fingerprint(input_hashes: &[String]) -> String {
    let mut input_hashes_sorted = input_hashes.to_vec();
    input_hashes_sorted.sort();
    input_hashes_sorted.dedup();
    let mut hasher = sha2::Sha256::new();
    for hash in input_hashes_sorted {
        hasher.update(hash.as_bytes());
        hasher.update(b",");
    }
    format!("{:x}", hasher.finalize())
}
