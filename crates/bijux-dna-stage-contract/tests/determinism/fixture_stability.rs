use bijux_dna_testkit::{load_fixture_json, stable_json};

#[path = "../support/workspace_paths.rs"]
mod support;

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let fixture = support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("tests/fixtures/stage_contract_schema/default/stage_contract_schema.json");
    let value = load_fixture_json(fixture);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(sorted, resorted, "fixture JSON must be deterministically ordered");
}
