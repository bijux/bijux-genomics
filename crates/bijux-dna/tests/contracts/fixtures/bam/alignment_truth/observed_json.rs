#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_bam_alignment_truth_reports_governed_alignment_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "bam-alignment-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_alignment_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("bam-alignment-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-alignment-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-alignment-truth/expected.json")
    );
    assert_eq!(payload.get("validated_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("validated_record_count").and_then(serde_json::Value::as_u64), Some(14));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
