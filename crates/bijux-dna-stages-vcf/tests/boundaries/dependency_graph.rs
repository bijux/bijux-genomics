use std::collections::BTreeSet;
use std::path::PathBuf;

const NORMAL_DEPS: &[&str] = &[
    "anyhow",
    "bijux-dna-core",
    "bijux-dna-db-ref",
    "bijux-dna-domain-vcf",
    "bijux-dna-infra",
    "regex",
    "serde",
    "serde_json",
    "sha2",
];

const DEV_DEPS: &[&str] = &["bijux-dna-policies", "bijux-dna-testkit", "tempfile"];

const FORBIDDEN_BIJUX_EDGES: &[&str] = &[
    "bijux-dna-api",
    "bijux-dna-engine",
    "bijux-dna-environment",
    "bijux-dna-environment-qa",
    "bijux-dna-planner-bam",
    "bijux-dna-planner-fastq",
    "bijux-dna-planner-vcf",
    "bijux-dna-runner",
    "bijux-dna-runtime",
];

#[test]
fn normal_dependency_graph_matches_stages_vcf_boundary() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dependencies"),
        set(NORMAL_DEPS),
        "normal dependencies must match docs/DEPENDENCIES.md"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dev-dependencies"),
        set(DEV_DEPS),
        "dev dependencies must stay limited to policy, testkit, and tempfile support"
    );
}

#[test]
fn stages_vcf_rejects_api_planner_runtime_runner_and_environment_edges() {
    let manifest = manifest();

    for forbidden in FORBIDDEN_BIJUX_EDGES {
        assert!(
            !manifest.contains(forbidden),
            "stages-vcf must not depend on orchestration crate {forbidden}"
        );
    }
}

fn manifest() -> String {
    std::fs::read_to_string(crate_root().join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"))
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn section_keys(manifest: &str, section: &str) -> BTreeSet<String> {
    let header = format!("[{section}]");
    let mut in_section = false;
    let mut keys = BTreeSet::new();

    for line in manifest.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
            continue;
        }

        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let key = trimmed.split_once('=').map_or(trimmed, |(key, _)| key.trim());
        let key = key.strip_suffix(".workspace").unwrap_or(key).to_string();
        keys.insert(key);
    }

    keys
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
