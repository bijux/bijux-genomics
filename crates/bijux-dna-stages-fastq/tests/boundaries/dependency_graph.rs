#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn normal_dependency_graph_matches_stages_fastq_boundary() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");
    let dependencies = dependency_names(&manifest, "dependencies");
    let expected = entries([
        "anyhow",
        "bijux-dna-core",
        "bijux-dna-domain-fastq",
        "bijux-dna-infra",
        "bijux-dna-stage-contract",
        "flate2",
        "serde",
        "serde_json",
        "tracing",
    ]);

    assert_eq!(
        dependencies, expected,
        "normal dependencies must stay limited to FASTQ stage contracts"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");
    let dependencies = dependency_names(&manifest, "dev-dependencies");
    let expected = entries([
        "anyhow",
        "bijux-dna-policies",
        "bijux-dna-testkit",
        "serde_json",
        "sha2",
        "tempfile",
        "walkdir",
    ]);

    assert_eq!(dependencies, expected, "dev dependencies must stay test-facing");
}

#[test]
fn internal_dependencies_use_workspace_catalog() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");

    for dependency in
        ["bijux-dna-core", "bijux-dna-domain-fastq", "bijux-dna-infra", "bijux-dna-stage-contract"]
    {
        assert!(
            manifest.contains(&format!("{dependency}.workspace = true")),
            "`{dependency}` must come from the workspace catalog"
        );
    }
    assert!(
        !manifest.contains("path = \"../bijux-dna-"),
        "focused FASTQ stage dependencies must not use ad hoc internal path declarations"
    );
}

#[test]
fn stages_fastq_rejects_planner_runtime_runner_api_and_environment_edges() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");
    let dependencies = dependency_names(&manifest, "dependencies");
    let forbidden = [
        "bijux-dna",
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-runtime",
    ];

    for dependency in forbidden {
        assert!(
            !dependencies.contains(dependency),
            "bijux-dna-stages-fastq must not depend on `{dependency}`"
        );
    }
}

fn dependency_names(manifest: &str, section: &str) -> BTreeSet<String> {
    let section_header = format!("[{section}]");
    let mut names = BTreeSet::new();
    let mut in_section = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line == section_header {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with('[') {
            break;
        }
        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, _)) = line.split_once('=') else {
            continue;
        };
        let name =
            name.trim().trim_matches('"').split_once('.').map_or(name.trim(), |(name, _)| name);
        names.insert(name.to_string());
    }

    names
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
