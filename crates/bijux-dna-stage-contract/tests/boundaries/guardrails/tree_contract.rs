#[test]
fn tree_contract_is_minimal() {
    let root = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let expected = ["Cargo.toml", "PUBLIC_API.md", "README.md", "docs/", "src/", "tests/"];
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read crate root") {
        let entry = entry.expect("read entry");
        let path = entry.path();
        let name = if path.is_dir() {
            format!("{}/", entry.file_name().to_string_lossy())
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        entries.push(name);
    }
    entries.sort();
    let expected_set: std::collections::BTreeSet<_> =
        expected.iter().map(|s| s.to_string()).collect();
    let entries_set: std::collections::BTreeSet<_> = entries.into_iter().collect();
    assert_eq!(
        entries_set, expected_set,
        "Stage-contract tree must remain minimal; update tree contract intentionally."
    );

    let src_dir = root.join("src");
    let allowed_src = [
        "execution_plan/",
        "lib.rs",
        "executor_registry/",
        "plan_run/",
        "stage_plan/",
        "stage_plugin.rs",
    ];
    let mut src_entries = Vec::new();
    for entry in std::fs::read_dir(&src_dir).expect("read src dir") {
        let entry = entry.expect("read src entry");
        let path = entry.path();
        let name = if path.is_dir() {
            format!("{}/", entry.file_name().to_string_lossy())
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        src_entries.push(name);
    }
    src_entries.sort();
    let allowed_set: std::collections::BTreeSet<_> =
        allowed_src.iter().map(|s| s.to_string()).collect();
    let src_set: std::collections::BTreeSet<_> = src_entries.into_iter().collect();
    assert_eq!(
        src_set, allowed_set,
        "Stage-contract src must match the contract architecture layout."
    );
}
