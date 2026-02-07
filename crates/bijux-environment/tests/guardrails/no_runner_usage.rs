use walkdir::WalkDir;

#[test]
fn environment_does_not_use_runner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if content.contains("bijux_runner") || content.contains("Runner") {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-environment must not depend on runner execution:\n{}",
        offenders.join("\n")
    );
}
