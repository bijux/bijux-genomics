use bijux_testkit::{load_fixture_json, stable_json};
use std::path::PathBuf;

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stage_contract_schema/default/stage_contract_schema.json");
    let value = load_fixture_json(fixture);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
