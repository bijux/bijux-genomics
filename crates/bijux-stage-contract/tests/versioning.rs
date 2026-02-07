#[test]
fn contract_versioning_rules_documented() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("CONTRACT.md");
    let content = std::fs::read_to_string(&path).expect("read CONTRACT.md");
    assert!(content.contains("Breaking"), "CONTRACT.md missing breaking-change rules");
    assert!(content.contains("major"), "CONTRACT.md missing major bump guidance");
}
