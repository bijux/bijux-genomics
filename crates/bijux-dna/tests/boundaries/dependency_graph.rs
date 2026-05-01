use std::collections::BTreeSet;
use std::path::PathBuf;

const NORMAL_DEPS: &[&str] = &[
    "anyhow",
    "bijux-dna-analyze",
    "bijux-dna-api",
    "bijux-dna-db-ena",
    "bijux-dna-domain-compiler",
    "bijux-dna-infra",
    "clap",
    "flate2",
    "regex",
    "serde",
    "serde_json",
    "sha2",
    "tar",
    "toml",
    "tracing",
];

const DEV_DEPS: &[&str] = &["bijux-dna-policies", "filetime", "insta", "predicates", "tempfile"];

#[test]
fn normal_dependency_graph_stays_cli_facing() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dependencies"),
        set(NORMAL_DEPS),
        "CLI normal dependencies must remain the documented adapter surface"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest = manifest();

    assert_eq!(
        section_keys(&manifest, "dev-dependencies"),
        set(DEV_DEPS),
        "CLI dev dependencies must remain test and policy support only"
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
