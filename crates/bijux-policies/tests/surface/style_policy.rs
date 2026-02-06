#[path = "../support/fs.rs"]
mod support;

#[test]
fn scope_docs_reference_workspace_style() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let scope_path = crate_root.join("SCOPE.md");
        if !scope_path.exists() {
            continue;
        }
        let content = support::read_to_string(&scope_path);
        if !content.contains("STYLE.md") {
            offenders.push(scope_path.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "SCOPE.md must link to STYLE.md:\n{}",
        offenders.join("\n")
    );
}
