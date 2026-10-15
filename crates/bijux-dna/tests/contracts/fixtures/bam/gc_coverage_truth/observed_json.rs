#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_bam_gc_coverage_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "bam-gc-coverage-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_gc_coverage_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("bam-gc-coverage-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-gc-coverage-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/bam-gc-coverage-truth/expected.json")
    );
    assert_eq!(
        payload.get("validated_coverage_sample_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_gc_bias_sample_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_coverage_region_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload.get("validated_gc_bias_bin_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
