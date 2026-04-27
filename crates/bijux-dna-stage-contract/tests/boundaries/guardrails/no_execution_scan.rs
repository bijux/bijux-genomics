#[test]
fn no_execution_details() {
    let root = crate::support::crate_root("bijux-dna-stage-contract")
        .map_or_else(|err| panic!("resolve crate src: {err}"), |root| root.join("src"));
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        if content.contains(concat!("Command::", "new"))
            || content.contains("docker")
            || content.contains("RuntimeKind")
            || content.contains(concat!("std::process::", "Command"))
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
