use walkdir::WalkDir;

#[test]
fn runtime_emits_use_canonical_json_writer() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let patterns = [
        "serde_json::to_writer",
        "serde_json::to_writer_pretty",
        "serde_json::to_string_pretty",
        "serde_json::to_vec_pretty",
        "atomic_write_json(",
    ];
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if patterns.iter().any(|p| content.contains(p)) {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "Runtime must use canonical JSON writer for emits.\nOffenders:\n{}",
        offenders.join("\n")
    );
}
