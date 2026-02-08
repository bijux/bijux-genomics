use serde_json::Value;
use std::path::PathBuf;

use std::fs;

fn load_fixture(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse fixture json: {err}"))
}

#[test]
fn contract_handshake_rejects_mismatched_schema() {
    let policies_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| panic!("crates dir missing"))
        .join("bijux-policies")
        .join("tests")
        .join("fixtures")
        .join("handshake")
        .join("default");
    let plan = load_fixture(&policies_root.join("plan.json"));
    let manifest = load_fixture(&policies_root.join("manifest.json"));
    let report = load_fixture(&policies_root.join("report.json"));

    assert!(plan.get("schema_version").is_some());
    assert!(manifest.get("schema_version").is_some());
    assert!(report.get("schema_version").is_some());
}
