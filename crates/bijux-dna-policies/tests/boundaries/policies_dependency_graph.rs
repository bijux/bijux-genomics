#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

#[test]
fn policy__boundaries__policies_dependency_graph__dependency_graph_matches_policy_crate_boundary() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));
    let manifest = manifest
        .parse::<Value>()
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest_path.display()));

    assert_eq!(
        dependency_keys(&manifest, "dependencies"),
        keys(["anyhow", "regex", "serde", "toml", "walkdir"]),
        "runtime dependencies must stay limited to deterministic policy support"
    );

    assert_eq!(
        dependency_keys(&manifest, "dev-dependencies"),
        keys([
            "bijux-dna-core",
            "bijux-dna-pipelines",
            "bijux-dna-runtime",
            "bijux-dna-stage-contract",
            "bijux-dna-testkit",
            "cargo_metadata",
            "insta",
            "serde_json",
            "serde_yaml",
            "sha2",
        ]),
        "dev dependencies must carry repository inspection and contract test support"
    );

    let runtime_dependencies = dependency_keys(&manifest, "dependencies");
    for forbidden in [
        "cargo_metadata",
        "serde_json",
        "serde_yaml",
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-engine",
        "bijux-dna-runtime",
        "bijux-dna-runner",
        "bijux-dna-pipelines",
    ] {
        assert!(
            !runtime_dependencies.contains(forbidden),
            "{forbidden} must stay outside the bijux-dna-policies runtime dependency graph"
        );
    }
}

fn dependency_keys(manifest: &Value, section: &str) -> BTreeSet<String> {
    manifest
        .get(section)
        .and_then(Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

fn keys<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
