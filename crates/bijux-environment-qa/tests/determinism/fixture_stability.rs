use bijux_testkit::{load_fixture_json, stable_json};

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let value = load_fixture_json("crates/bijux-environment-qa/tests/fixtures/qa_artifacts/default/report.json");
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(sorted, resorted, "fixture JSON must be deterministically ordered");
}
