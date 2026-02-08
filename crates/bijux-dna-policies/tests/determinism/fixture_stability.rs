#![allow(non_snake_case)]

use bijux_dna_testkit::{load_fixture_json, stable_json};
use std::path::PathBuf;

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn policy__determinism__fixture_stability__fixture_json_is_stable() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/handshake/default/plan.json");
    let value = load_fixture_json(fixture);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
