use std::collections::BTreeMap;

#[test]
fn stage_contracts_snapshot() {
    let mut contracts = BTreeMap::new();
    for stage in bijux_stages_fastq::implemented_stages() {
        let contract = bijux_stages_fastq::contracts::contract_for_stage(stage.as_str())
            .expect("contract");
        contracts.insert(stage.as_str().to_string(), contract);
    }
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&contracts)
            .expect("canonical"),
    )
    .expect("utf8");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stage_contracts.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).expect("write snapshot");
    }
    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    assert_eq!(actual, expected);
}
