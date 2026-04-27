use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

#[test]
fn dependency_graph_matches_pipeline_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let manifest =
        manifest.parse::<Value>().unwrap_or_else(|err| panic!("parse Cargo.toml: {err}"));

    assert_eq!(
        dependency_keys(&manifest, "dependencies"),
        keys([
            "anyhow",
            "bijux-dna-core",
            "bijux-dna-domain-bam",
            "bijux-dna-domain-fastq",
            "bijux-dna-domain-vcf",
            "serde",
            "serde_json",
            "sha2",
            "toml",
        ]),
        "runtime dependencies must stay limited to core/domain contracts and serialization"
    );
    assert_eq!(
        dependency_keys(&manifest, "dev-dependencies"),
        keys([
            "bijux-dna-policies",
            "bijux-dna-runtime",
            "bijux-dna-testkit",
            "insta",
            "walkdir",
        ]),
        "test dependencies must stay limited to policies, snapshots, test fixtures, and downstream model checks"
    );

    let all_dependencies = all_dependency_keys(&manifest);
    for forbidden in [
        "bijux-dna",
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-db-ena",
        "bijux-dna-db-ref",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-science",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ] {
        assert!(
            !all_dependencies.contains(forbidden),
            "{forbidden} must stay downstream of bijux-dna-pipelines"
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
