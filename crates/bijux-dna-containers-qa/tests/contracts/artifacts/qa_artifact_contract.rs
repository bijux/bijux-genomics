#[test]
fn qa_artifacts_follow_manifest_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/qa_artifacts/default");
    let manifest = root.join("manifest.json");
    let report = root.join("report.json");
    assert!(manifest.exists(), "missing manifest.json fixture");
    assert!(report.exists(), "missing report.json fixture");
    let manifest_value: serde_json::Value = {
        let content = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
        serde_json::from_str(&content).unwrap_or_else(|err| panic!("manifest json: {err}"))
    };
    assert!(manifest_value.get("schema_version").is_some());
    let report_value: serde_json::Value = {
        let content = std::fs::read_to_string(&report)
            .unwrap_or_else(|err| panic!("read {}: {err}", report.display()));
        serde_json::from_str(&content).unwrap_or_else(|err| panic!("report json: {err}"))
    };
    assert!(report_value.get("schema_version").is_some());
}
