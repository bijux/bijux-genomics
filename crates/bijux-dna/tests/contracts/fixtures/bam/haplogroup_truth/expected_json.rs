#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_expected_bam_haplogroup_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate-expected", "--corpus", "haplogroup-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_haplogroup_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("haplogroup-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/haplogroup-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/haplogroup-truth/expected.json")
    );
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("validated_ready_case_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_uncertain_case_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_coverage_gate_case_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_statuses").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("coverage_gate_not_met"),
            serde_json::json!("ready"),
            serde_json::json!("uncertain_marker_support"),
        ])
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
