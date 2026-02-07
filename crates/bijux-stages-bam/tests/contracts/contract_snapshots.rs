use std::collections::BTreeMap;

#[test]
fn stage_contracts_snapshot() {
    let mut contracts = BTreeMap::new();
    for stage in bijux_stages_bam::implemented_stages() {
        let contract = bijux_domain_bam::contract_for_stage(stage.as_str())
            .unwrap_or_else(|| panic!("contract missing for {}", stage.as_str()));
        contracts.insert(stage.as_str().to_string(), contract);
    }
    let actual = {
        let mut lines = Vec::new();
        for (stage, contract) in &contracts {
            lines.push(format!("{stage}: {contract:?}"));
        }
        lines.sort();
        lines.join("\n")
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stage_contracts.json");
    if std::env::var("UPDATE_CONTRACTS").ok().as_deref() == Some("1") {
        std::fs::write(&path, &actual).unwrap_or_else(|err| panic!("write snapshot: {err}"));
    }
    let expected =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read snapshot: {err}"));
    assert_eq!(actual, expected);
}
