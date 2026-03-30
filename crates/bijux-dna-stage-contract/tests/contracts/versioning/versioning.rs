#[test]
fn contract_versioning_rules_documented() {
    let path = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("CONTRACT.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read docs/CONTRACT.md failed: {err}"));
    assert!(
        content.contains("Breaking"),
        "CONTRACT.md missing breaking-change rules"
    );
    assert!(
        content.contains("major"),
        "CONTRACT.md missing major bump guidance"
    );
}
