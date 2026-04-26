use std::collections::BTreeSet;
use std::path::PathBuf;

const NORMAL_DEPS: &[&str] = &[
    "anyhow",
    "bijux-dna-analyze",
    "bijux-dna-core",
    "bijux-dna-domain-bam",
    "bijux-dna-domain-fastq",
    "bijux-dna-domain-vcf",
    "bijux-dna-engine",
    "bijux-dna-environment",
    "bijux-dna-infra",
    "bijux-dna-pipelines",
    "bijux-dna-planner-bam",
    "bijux-dna-planner-fastq",
    "bijux-dna-policies",
    "bijux-dna-runner",
    "bijux-dna-runtime",
    "bijux-dna-stage-contract",
    "bijux-dna-stages-vcf",
    "cargo_metadata",
    "chrono",
    "flate2",
    "regex",
    "serde",
    "serde_json",
    "sha2",
    "toml",
    "tracing",
    "uuid",
    "walkdir",
];

const DEV_DEPS: &[&str] = &["bijux-dna-testkit", "insta", "tempfile"];

#[test]
fn normal_dependency_graph_matches_api_integration_boundary() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dependencies"),
        set(NORMAL_DEPS),
        "API dependencies must stay explicit because this crate is the integration boundary"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dev-dependencies"),
        set(DEV_DEPS),
        "API dev dependencies must remain test support only"
    );
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
    let mut keys = BTreeSet::new();
    let mut in_section = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == header;
            continue;
        }
        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, _value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        keys.insert(key.strip_suffix(".workspace").unwrap_or(key).to_string());
    }

    keys
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
