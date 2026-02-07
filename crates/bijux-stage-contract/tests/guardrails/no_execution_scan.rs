#[test]
fn no_execution_details() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("Command::new")
            || content.contains("docker")
            || content.contains("RunnerKind")
            || content.contains("std::process::Command")
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "Stage-contract must not reference execution details (runner/env/docker).\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
