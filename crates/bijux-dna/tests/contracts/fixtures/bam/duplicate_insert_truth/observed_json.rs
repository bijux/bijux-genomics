#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_bam_duplicate_insert_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "bam-duplicate-insert-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_duplicate_insert_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("bam-duplicate-insert-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-duplicate-insert-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-duplicate-insert-truth/expected.json")
    );
    assert_eq!(payload.get("validated_sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("validated_duplicate_family_bin_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        payload.get("validated_insert_size_bin_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
