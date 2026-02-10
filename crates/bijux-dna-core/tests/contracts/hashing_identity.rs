use bijux_dna_core::prelude::cache::CacheKey;
use bijux_dna_core::prelude::hashing::{input_fingerprint, run_id_from_hashes};

#[test]
fn run_id_is_order_independent_for_input_hashes() {
    let a = vec!["sha256:b".to_string(), "sha256:a".to_string()];
    let b = vec!["sha256:a".to_string(), "sha256:b".to_string()];
    let run_a = run_id_from_hashes("pipe", "sample", "params", &a, None);
    let run_b = run_id_from_hashes("pipe", "sample", "params", &b, None);
    assert_eq!(run_a, run_b);
}

#[test]
fn input_fingerprint_is_order_independent_and_deduped() {
    let a = vec![
        "sha256:b".to_string(),
        "sha256:a".to_string(),
        "sha256:a".to_string(),
    ];
    let b = vec!["sha256:a".to_string(), "sha256:b".to_string()];
    assert_eq!(input_fingerprint(&a), input_fingerprint(&b));
}

#[test]
fn cache_key_identity_tuple_is_explicit_and_stable() {
    let key = CacheKey::new("in", "params", "tool@1", "sha256:env");
    assert_eq!(key.as_string(), "in|params|tool@1|sha256:env");
    let payload = serde_json::to_string(&key).expect("serialize cache key");
    assert!(payload.contains("input_fingerprint"));
    assert!(payload.contains("parameters_fingerprint"));
    assert!(payload.contains("tool_version"));
    assert!(payload.contains("env_digest"));
}
