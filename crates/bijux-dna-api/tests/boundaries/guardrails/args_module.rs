use walkdir::WalkDir;

#[test]
fn args_module_is_named_request_args() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().file_name().and_then(|n| n.to_str()) == Some("args.rs") {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "args.rs is forbidden; use request_args.rs only.\nOffenders:\n{}",
        offenders.join("\n")
    );
}
