use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn load_fixture_json(path: &Path) -> Value {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    serde_json::from_str(&content).unwrap_or_else(|err| panic!("parse fixture json: {err}"))
}

fn stable_json(value: &Value) -> Value {
    bijux_dna_core::contract::canonical::canonicalize_truth_json(value)
}

/// Ensures JSON fixtures serialize deterministically.
#[test]
fn fixture_json_is_stable() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/runtime_schema/default/run_record_v1.json");
    let value = load_fixture_json(&fixture);
    let sorted = stable_json(&value);
    let resorted = stable_json(&sorted);
    assert_eq!(
        sorted, resorted,
        "fixture JSON must be deterministically ordered"
    );
}
