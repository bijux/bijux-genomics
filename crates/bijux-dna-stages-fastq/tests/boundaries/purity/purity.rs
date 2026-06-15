#![allow(clippy::expect_used)]

#[test]
fn no_command_building_or_tool_selection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("CommandSpecV1")
            || content.contains("ContainerImageRefV1")
            || content.contains("ToolId::new(\"")
            || content.contains("StageId::new(\"")
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "Stage crate must not build commands or select tools; only specs/observers allowed.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn stage_specs_are_declarative() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("stage_specs")
        .join("mod.rs");
    let contents = std::fs::read_to_string(&path).expect("read stage_specs/mod.rs");
    let banned = ["std::process", "Command::", "tokio::process", "shell", "exec"];
    for needle in banned {
        assert!(
            !contents.contains(needle),
            "stage_specs/mod.rs must be declarative; found {needle}"
        );
    }
}
