#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_expected_adna_contamination_truth_reports_governed_bundle() {
    let payload = run_cli_json(&[
        "fixtures",
        "validate-expected",
        "--corpus",
        "adna-contamination-truth",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.adna_contamination_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("adna-contamination-truth")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/adna-contamination-truth/expected.json")
    );
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        payload.get("validated_insufficient_case_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
