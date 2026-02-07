use walkdir::WalkDir;

#[test]
fn engine_is_orchestrator_only() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let patterns = [
        "std::process::Command",
        "Command::new",
        "docker",
        "apptainer",
        "singularity",
    ];
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read source: {err}"));
        if patterns.iter().any(|p| content.contains(p)) {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "Engine must not spawn processes or reference docker/local execution.\n\
Move execution to bijux-runner.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
