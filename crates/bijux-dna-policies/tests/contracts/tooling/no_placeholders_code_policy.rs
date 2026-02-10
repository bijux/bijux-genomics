#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__no_placeholders_code_policy__production_paths_ban_todo_unimplemented() {
    let root = support::workspace_root();
    let crates = root.join("crates");
    let mut offenders = Vec::new();

    for entry in walkdir::WalkDir::new(&crates).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).to_string_lossy();
        if rel.contains("/tests/")
            || rel.contains("/benches/")
            || rel.contains("/examples/")
            || rel.ends_with("/main.rs")
        {
            continue;
        }
        let raw = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("read source file {}", path.display()));
        if raw.contains("todo!(")
            || raw.contains("unimplemented!(")
            || raw.contains("bail!(\"not implemented")
        {
            offenders.push(rel.to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "placeholder code detected in production paths:\n{}",
        offenders.join("\n")
    );
}
