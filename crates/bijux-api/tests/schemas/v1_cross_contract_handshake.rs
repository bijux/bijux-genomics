use serde_json::Value;
use std::path::PathBuf;

use std::fs;

fn load_fixture(path: PathBuf) -> Value {
    let raw = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&raw).expect("parse fixture json")
}

#[test]
fn contract_handshake_rejects_mismatched_schema() {
    let policies_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .join("bijux-policies")
        .join("tests")
        .join("fixtures")
        .join("handshake")
        .join("default");
    let plan = load_fixture(
        policies_root.join("plan.json"),
    );
    let manifest = load_fixture(
        policies_root.join("manifest.json"),
    );
    let report = load_fixture(
        policies_root.join("report.json"),
    );

    assert!(plan.get("schema_version").is_some());
    assert!(manifest.get("schema_version").is_some());
    assert!(report.get("schema_version").is_some());
}
