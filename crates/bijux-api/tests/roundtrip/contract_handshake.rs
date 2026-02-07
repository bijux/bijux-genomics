use std::path::PathBuf;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn load_fixture(path: &str) -> Value {
    let file = workspace_root().join(path);
    let raw = std::fs::read_to_string(&file).expect("read fixture");
    serde_json::from_str(&raw).expect("parse fixture JSON")
}

#[test]
fn contract_handshake_fixtures_parse() {
    let plan = load_fixture("crates/bijux-policies/tests/fixtures/handshake/plan.json");
    let manifest = load_fixture("crates/bijux-policies/tests/fixtures/handshake/manifest.json");
    let report = load_fixture("crates/bijux-policies/tests/fixtures/handshake/report.json");

    assert_eq!(plan["schema_version"], "bijux.plan.v1");
    assert!(plan.get("steps").is_some());

    assert_eq!(manifest["schema_version"], "bijux.manifest.v1");
    assert!(manifest.get("artifacts").is_some());

    assert_eq!(report["schema_version"], "bijux.report.v1");
    assert!(report.get("summary").is_some());
}
