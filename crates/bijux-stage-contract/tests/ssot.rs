#[test]
fn stage_contract_types_are_ssot() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/crates/bijux-stage-contract/") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("struct StagePlanV1")
            || content.contains("struct ExecutionPlan")
            || content.contains("struct StagePluginOutputV1")
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "Stage-contract types must be defined only in bijux-stage-contract.\n\
Move definitions into this crate and import them elsewhere.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
