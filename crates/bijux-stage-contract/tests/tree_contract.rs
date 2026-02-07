#[test]
fn tree_contract_is_minimal() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = [
        "Cargo.toml",
        "CONTRACT.md",
        "SCOPE.md",
        "ARCHITECTURE.md",
        "src/",
        "tests/",
    ];
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(root).expect("read crate root") {
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
    let expected_set: std::collections::BTreeSet<_> = expected.iter().map(|s| s.to_string()).collect();
    let entries_set: std::collections::BTreeSet<_> = entries.into_iter().collect();
    assert_eq!(
        entries_set, expected_set,
        "Stage-contract tree must remain minimal; update tree contract intentionally."
    );
}
