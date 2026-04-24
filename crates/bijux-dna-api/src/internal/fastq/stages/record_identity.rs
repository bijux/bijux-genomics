use sha2::{Digest, Sha256};
use std::fmt::Write as _;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

/// Derive a deterministic parameter fingerprint even when canonical hashing fails.
///
/// This keeps benchmark identity stable across repeated runs instead of falling back
/// to a random UUID when the raw parameter payload is not canonically hashable.
#[must_use]
pub fn stable_params_hash(params: &serde_json::Value) -> String {
    bijux_dna_core::prelude::params_hash(params).unwrap_or_else(|hash_error| {
        let raw = serde_json::to_vec(params).unwrap_or_else(|serialize_error| {
            format!("unhashable_params|hash_error={hash_error}|serialize_error={serialize_error}")
                .into_bytes()
        });
        let mut hasher = Sha256::new();
        hasher.update(raw);
        format!("fallback:{}", sha256_hex(&hasher.finalize()))
    })
}

#[cfg(test)]
mod tests {
    use super::stable_params_hash;

    #[test]
    fn stable_params_hash_is_deterministic_for_repeated_payloads() {
        let params = serde_json::json!({
            "quality_cutoff": 20,
            "tool": "fastp"
        });

        let first = stable_params_hash(&params);
        let second = stable_params_hash(&params);

        assert_eq!(first, second);
        assert!(!first.is_empty());
    }
}
