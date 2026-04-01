use std::collections::BTreeMap;

use bijux_dna_core::metrics::ToolInvocationV1;
use bijux_dna_core::prelude::hashing::params_hash;
use sha2::Digest;

fn canonical_params_hash(value: &serde_json::Value) -> Option<String> {
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(value).ok()?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn empty_params_hash() -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"{}");
    format!("{:x}", hasher.finalize())
}

pub(super) fn resolved_params_hash(
    key: &str,
    params_hashes: &BTreeMap<String, String>,
    invocation: &ToolInvocationV1,
) -> String {
    params_hashes
        .get(key)
        .cloned()
        .or_else(|| params_hash(&invocation.parameters_json_normalized).ok())
        .or_else(|| canonical_params_hash(&invocation.effective_params_json_normalized))
        .or_else(|| canonical_params_hash(&invocation.parameters_json_normalized))
        .unwrap_or_else(empty_params_hash)
}
