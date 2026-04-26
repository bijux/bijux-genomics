use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

#[test]
fn dependency_graph_matches_planner_vcf_boundary() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));
    let manifest = manifest
        .parse::<Value>()
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest_path.display()));

    assert_eq!(
        dependency_keys(&manifest, "dependencies"),
        keys([
            "anyhow",
            "bijux-dna-core",
            "bijux-dna-db-ref",
            "bijux-dna-domain-vcf",
            "bijux-dna-stage-contract",
            "serde",
            "serde_json",
            "sha2",
            "toml",
        ]),
        "runtime dependencies must stay limited to planning contracts, VCF domain vocabulary, reference catalog handoff, serialization, hashing, and registry parsing"
    );

    assert_eq!(
        dependency_keys(&manifest, "dev-dependencies"),
        keys(["bijux-dna-policies"]),
        "dev dependencies must stay limited to guardrail policy checks"
    );

    let all_dependencies = all_dependency_keys(&manifest);
    for forbidden in [
        "bijux-dna",
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-bench-model",
        "bijux-dna-cli",
        "bijux-dna-dev",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-science",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
        "reqwest",
        "ureq",
    ] {
        assert!(
            !all_dependencies.contains(forbidden),
            "{forbidden} must stay outside the bijux-dna-planner-vcf dependency graph"
        );
    }
}

fn all_dependency_keys(manifest: &Value) -> BTreeSet<String> {
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .into_iter()
        .flat_map(|section| dependency_keys(manifest, section))
        .collect()
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
