use std::path::PathBuf;

use bijux_dna_testkit::{load_fixture_json, stable_json};

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/env_schema/default/tool_image_spec.json");
    let value = load_fixture_json(&fixture_path);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
