use serde_json::Value;
use std::fs;

use bijux_dna_api::v1::api::bench::{
    render_benchmark_attempt_schema, render_benchmark_cycle_status_schema,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
};

fn load_fixture(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse fixture json: {err}"))
}

#[test]
fn contract_handshake_rejects_mismatched_schema() {
    let policies_root = crate::support::crate_root("bijux-dna-policies")
        .unwrap_or_else(|err| panic!("resolve policies crate root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("handshake")
        .join("default");
    let plan = load_fixture(&policies_root.join("plan.json"));
    let manifest = load_fixture(&policies_root.join("manifest.json"));
    let report = load_fixture(&policies_root.join("report.json"));

    assert!(plan.get("schema_version").is_some());
    assert!(manifest.get("schema_version").is_some());
    assert!(report.get("schema_version").is_some());
}

#[test]
fn benchmark_cycle_schemas_are_governed_by_rust_types() {
    let attempt = render_benchmark_attempt_schema();
    let cycle_status = render_benchmark_cycle_status_schema();

    assert_eq!(
        attempt
            .get("properties")
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some(BENCHMARK_ATTEMPT_SCHEMA_V1),
    );
    assert_eq!(
        cycle_status
            .get("properties")
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some(BENCHMARK_CYCLE_STATUS_SCHEMA_V1),
    );
}
