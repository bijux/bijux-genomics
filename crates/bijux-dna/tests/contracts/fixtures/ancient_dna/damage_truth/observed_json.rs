#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli_json;

#[test]
fn fixtures_validate_adna_damage_truth_reports_governed_bundle() {
    let payload =
        run_cli_json(&["fixtures", "validate", "--corpus", "adna-damage-truth", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.adna_damage_truth.validation.v1")
    );
    assert_eq!(
        payload.get("fixture_id").and_then(serde_json::Value::as_str),
        Some("adna-damage-truth")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/adna-damage-truth/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/science/adna-damage-truth/expected.json")
    );
    assert_eq!(payload.get("validated_case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("validated_insufficient_case_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let classes = payload
        .get("terminal_pattern_classes")
        .and_then(serde_json::Value::as_array)
        .expect("terminal pattern classes");
    assert!(classes.iter().any(|value| value.as_str() == Some("ct5p_dominant")));
    assert!(classes.iter().any(|value| value.as_str() == Some("insufficient_terminal_damage")));
}
