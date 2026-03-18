#[test]
fn guardrails() {
    let snapshot = bijux_dna_testkit::snapshot_name("schemas", "public_api");
    assert_eq!(snapshot, "bijux-dna-testkit__schemas__public_api");
}
