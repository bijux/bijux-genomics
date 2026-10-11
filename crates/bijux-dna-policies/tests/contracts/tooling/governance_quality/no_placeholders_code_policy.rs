#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__no_placeholders_code_policy__production_paths_ban_todo_markers_and_unimplemented(
) {
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
        if contains_production_placeholder(&raw) {
            offenders.push(rel.to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "placeholder code detected in production paths:\n{}",
        offenders.join("\n")
    );
}

fn contains_production_placeholder(raw: &str) -> bool {
    let production = bijux_dna_testkit::workspace_support::strip_rust_test_modules(raw);
    production.contains("TODO")
        || production.contains("todo!(")
        || production.contains("unimplemented!(")
        || production.contains("bail!(\"not implemented")
}

#[test]
fn policy__contracts__no_placeholders_code_policy__reject_production_markers_and_allow_negative_fixtures(
) {
    for source in
        ["// TODO implement admission", "fn run() { todo!() }", "fn run() { unimplemented!() }"]
    {
        assert!(contains_production_placeholder(source));
    }
    let fixture = "fn run() {}\n#[cfg(test)]\nmod tests {\n    const REJECTED: &str = \"TODO run tool\";\n}\n";
    assert!(!contains_production_placeholder(fixture));
    assert!(contains_production_placeholder(&format!("{fixture}\nfn run_later() {{ todo!() }}")));
}
