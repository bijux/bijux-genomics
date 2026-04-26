use std::collections::BTreeSet;

#[test]
fn normal_dependency_graph_matches_stage_contract_boundary() {
    let manifest = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("Cargo.toml");
    let parsed = read_manifest(&manifest);
    let dependencies = dependency_names(&parsed, "dependencies");

    assert_eq!(
        dependencies,
        entries(["anyhow", "bijux-dna-core", "serde", "serde_json", "sha2"]),
        "stage-contract runtime dependencies must stay pure contract-facing"
    );
    for forbidden in [
        "bijux-dna-api",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-runner",
        "bijux-dna-runtime",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "stage-contract must not depend on execution or orchestration crate `{forbidden}`"
        );
    }
}

#[test]
fn dev_dependency_graph_stays_policy_and_fixture_facing() {
    let manifest = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("Cargo.toml");
    let parsed = read_manifest(&manifest);
    let dev_dependencies = dependency_names(&parsed, "dev-dependencies");

    assert_eq!(
        dev_dependencies,
        entries(["bijux-dna-policies", "bijux-dna-testkit", "toml", "walkdir"]),
        "stage-contract dev dependencies must stay limited to policies, testkit, manifest parsing, and source scans"
    );
}

fn read_manifest(path: &std::path::Path) -> toml::Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    toml::from_str(&content).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn dependency_names(parsed: &toml::Value, table_name: &str) -> BTreeSet<String> {
    parsed
        .get(table_name)
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

fn entries<const N: usize>(expected: [&str; N]) -> BTreeSet<String> {
    expected.into_iter().map(str::to_string).collect()
}
