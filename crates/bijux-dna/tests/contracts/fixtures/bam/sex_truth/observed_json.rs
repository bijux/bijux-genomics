#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_bam_sex_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "sex-inference-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_sex_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("sex-inference-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/sex-inference-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/sex-inference-truth/expected.json")
    );
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("validated_ok_case_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("validated_insufficient_case_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("validated_call_classes").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::json!("ambiguous"),
            serde_json::json!("female"),
            serde_json::json!("insufficient"),
            serde_json::json!("male"),
        ])
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
