use std::path::PathBuf;

use bijux_testkit::{load_fixture_json, stable_json};

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("bam")
        .join("default")
        .join("sex.json");
    let value = load_fixture_json(fixture_path);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
