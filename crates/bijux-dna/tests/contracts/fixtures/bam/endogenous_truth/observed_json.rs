#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_bam_endogenous_truth_reports_governed_bundle() {
    let payload = run_cli_json(&["fixtures", "validate", "--corpus", "endogenous-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_endogenous_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("endogenous-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/endogenous-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/endogenous-truth/expected.json")
    );
    assert_eq!(payload.get("validated_sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("validated_total_reads").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("validated_contaminant_reads").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload.get("validated_retained_reads").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
