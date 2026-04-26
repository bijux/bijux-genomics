use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

#[test]
fn dependency_graph_matches_runtime_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));
    let manifest = manifest
        .parse::<Value>()
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest_path.display()));
    let deps_doc = fs::read_to_string(root.join("docs/DEPENDENCIES.md"))
        .unwrap_or_else(|err| panic!("read docs/DEPENDENCIES.md: {err}"));

    assert_eq!(
        dependency_keys(&manifest, "dependencies"),
        keys([
            "anyhow",
            "bijux-dna-core",
            "bijux-dna-infra",
            "chrono",
            "opentelemetry",
            "serde",
            "serde_json",
            "sha2",
            "toml",
            "uuid",
        ]),
        "runtime dependencies must stay limited to contracts, governed I/O, identity, parsing, hashes, timestamps, and optional telemetry"
    );

    assert_eq!(
        dependency_keys(&manifest, "dev-dependencies"),
        keys(["bijux-dna-policies", "bijux-dna-testkit", "walkdir"]),
        "runtime dev dependencies must stay limited to policies, isolated fixtures, and tree scanning"
    );

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
        "bijux-dna-runner",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
        "reqwest",
        "tokio",
    ] {
        assert!(
            !dependency_keys(&manifest, "dependencies").contains(forbidden),
            "{forbidden} must stay outside the bijux-dna-runtime runtime dependency graph"
        );
        assert!(
            deps_doc.contains(forbidden) || forbidden.starts_with("bijux-dna-stages"),
            "docs/DEPENDENCIES.md should explain forbidden runtime family {forbidden}"
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
