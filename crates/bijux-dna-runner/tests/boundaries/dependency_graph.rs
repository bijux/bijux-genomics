use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

#[test]
fn dependency_graph_matches_runner_boundary() {
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
            "bijux-dna-environment",
            "bijux-dna-infra",
            "bijux-dna-runtime",
            "serde",
            "serde_json",
            "sha2",
            "tracing",
            "uuid",
            "walkdir",
        ]),
        "runtime dependencies must stay limited to execution contracts, environment resolution, infra effects, and deterministic identity"
    );

    assert_eq!(
        dependency_keys(&manifest, "dev-dependencies"),
        keys(["assert_cmd", "bijux-dna-policies", "cargo_metadata", "tempfile", "toml"]),
        "dev dependencies must stay limited to policy, metadata, command, and isolated filesystem tests"
    );

    let runtime_dependencies = dependency_keys(&manifest, "dependencies");
    for forbidden in [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-analyze",
        "bijux-dna-bench",
        "bijux-dna-engine",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
        "chrono",
        "flate2",
        "reqwest",
    ] {
        assert!(
            !runtime_dependencies.contains(forbidden),
            "{forbidden} must stay outside the bijux-dna-runner runtime dependency graph"
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
