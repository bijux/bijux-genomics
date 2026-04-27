use std::collections::BTreeSet;
use std::path::PathBuf;

const NORMAL_DEPS: &[&str] = &["rand", "serde_json", "tempfile"];
const DEV_DEPS: &[&str] = &["bijux-dna-policies"];

const FORBIDDEN_NORMAL_DEPS: &[&str] = &[
    "bijux-dna",
    "bijux-dna-analyze",
    "bijux-dna-api",
    "bijux-dna-bench",
    "bijux-dna-bench-model",
    "bijux-dna-core",
    "bijux-dna-db-ena",
    "bijux-dna-db-ref",
    "bijux-dna-domain-bam",
    "bijux-dna-domain-compiler",
    "bijux-dna-domain-fastq",
    "bijux-dna-domain-vcf",
    "bijux-dna-engine",
    "bijux-dna-environment",
    "bijux-dna-environment-qa",
    "bijux-dna-infra",
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

#[test]
fn normal_dependency_graph_stays_test_helper_only() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dependencies"),
        set(NORMAL_DEPS),
        "normal dependencies must match docs/DEPENDENCIES.md"
    );
}

#[test]
fn dev_dependency_graph_stays_policy_only() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dev-dependencies"),
        set(DEV_DEPS),
        "dev dependencies must stay limited to policy validation"
    );
}

#[test]
fn testkit_rejects_required_product_crate_edges() {
    let normal_deps = section_keys(&manifest(), "dependencies");

    for forbidden in FORBIDDEN_NORMAL_DEPS {
        assert!(
            !normal_deps.contains(*forbidden),
            "testkit must not require product workspace crate {forbidden}"
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
        keys.insert(key.strip_suffix(".workspace").unwrap_or(key).to_string());
    }

    keys
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
