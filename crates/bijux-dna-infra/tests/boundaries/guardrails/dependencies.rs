use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn runtime_dependencies_match_documented_boundary() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = manifest_section_keys(&manifest, "[dependencies]");
    let expected = BTreeSet::from([
        "fs4",
        "serde",
        "serde_json",
        "serde_yaml",
        "sha2",
        "tempfile",
        "thiserror",
        "toml",
        "tracing-appender",
        "tracing-subscriber",
    ]);

    assert_eq!(dependencies, expected, "runtime dependency boundary changed");

    let docs = fs::read_to_string(crate_root().join("docs/DEPENDENCIES.md"))
        .unwrap_or_else(|err| panic!("read docs/DEPENDENCIES.md: {err}"));
    for dependency in expected {
        assert!(docs.contains(dependency), "docs/DEPENDENCIES.md must mention {dependency}");
    }
}

#[test]
fn infra_does_not_depend_on_higher_level_workspace_crates() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = manifest_section_keys(&manifest, "[dependencies]");
    let forbidden = [
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-core",
        "bijux-dna-db-ena",
        "bijux-dna-db-ref",
        "bijux-dna-dev",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-compiler",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-science",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ];

    for name in forbidden {
        assert!(!dependencies.contains(name), "infra must not depend on {name}");
    }
}

#[test]
fn dev_dependencies_match_documented_test_boundary() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = manifest_section_keys(&manifest, "[dev-dependencies]");
    let expected = BTreeSet::from([
        "anyhow",
        "bijux-dna-policies",
        "bijux-dna-testkit",
        "insta",
        "regex",
        "walkdir",
    ]);

    assert_eq!(dependencies, expected, "infra dev dependency boundary changed");

    let docs = fs::read_to_string(crate_root().join("docs/DEPENDENCIES.md"))
        .unwrap_or_else(|err| panic!("read docs/DEPENDENCIES.md: {err}"));
    for dependency in expected {
        assert!(docs.contains(dependency), "docs/DEPENDENCIES.md must mention {dependency}");
    }
}

fn manifest_section_keys<'manifest>(
    manifest: &'manifest str,
    section: &str,
) -> BTreeSet<&'manifest str> {
    let mut in_section = false;
    let mut keys = BTreeSet::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            keys.insert(key.trim().split_once('.').map_or(key.trim(), |(name, _)| name));
        }
    }
    keys
}
