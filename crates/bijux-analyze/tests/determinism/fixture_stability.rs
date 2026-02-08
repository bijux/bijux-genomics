use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn load_fixture_json(path: PathBuf) -> Value {
    let raw = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&raw).expect("parse fixture json")
}

fn stable_json(value: &Value) -> Value {
    bijux_core::contract::canonical::canonicalize_truth_json(value)
}

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("bench_compare")
        .join("run-b")
        .join("metrics.json");
    let value = load_fixture_json(fixture_path);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
