#[test]
fn qa_artifacts_follow_manifest_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/qa_artifacts");
    let manifest = root.join("manifest.json");
    let report = root.join("report.json");
    assert!(manifest.exists(), "missing manifest.json fixture");
    assert!(report.exists(), "missing report.json fixture");
    let manifest_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest).expect("read"))
            .expect("manifest json");
    assert!(manifest_value.get("schema_version").is_some());
    let report_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).expect("read"))
            .expect("report json");
    assert!(report_value.get("schema_version").is_some());
}
