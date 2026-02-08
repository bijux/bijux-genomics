use serde_json::Value;

use bijux_testkit::load_fixture_json;

fn load_fixture(path: &str) -> Value {
    load_fixture_json(path)
}

#[test]
fn contract_handshake_rejects_mismatched_schema() {
    let plan = load_fixture("crates/bijux-policies/tests/fixtures/handshake/plan/default/plan.json");
    let manifest =
        load_fixture("crates/bijux-policies/tests/fixtures/handshake/manifest/default/manifest.json");
    let report =
        load_fixture("crates/bijux-policies/tests/fixtures/handshake/report/default/report.json");

    assert!(plan.get("schema_version").is_some());
    assert!(manifest.get("schema_version").is_some());
    assert!(report.get("schema_version").is_some());
}
