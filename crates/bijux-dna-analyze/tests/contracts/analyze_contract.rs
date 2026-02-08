use bijux_dna_analyze::analyze_contract_v1;

#[test]
fn analyze_contract_has_expected_version() {
    let contract = analyze_contract_v1();
    assert_eq!(contract.schema_version, "bijux.analyze_contract.v1");
    assert!(contract
        .supported_inputs
        .contains(&"facts.jsonl".to_string()));
    assert!(contract.outputs.contains(&"report.json".to_string()));
}
