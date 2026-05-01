use std::collections::BTreeSet;

#[test]
fn dependency_graph_matches_dev_boundary() {
    let root = crate::support::crate_root("bijux-dna-dev")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    let dependencies = section_keys(&manifest, "[dependencies]");
    let expected_dependencies: BTreeSet<_> = [
        "anyhow",
        "bijux-dna-api",
        "bijux-dna-core",
        "bijux-dna-db-ena",
        "bijux-dna-db-ref",
        "bijux-dna-infra",
        "chrono",
        "clap",
        "regex",
        "reqwest",
        "serde",
        "serde_json",
        "sha2",
        "toml",
        "walkdir",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(dependencies, expected_dependencies, "unexpected direct dependency shape");

    let dev_dependencies = section_keys(&manifest, "[dev-dependencies]");
    let expected_dev_dependencies: BTreeSet<_> =
        ["bijux-dna-policies"].into_iter().map(str::to_string).collect();
    assert_eq!(dev_dependencies, expected_dev_dependencies, "unexpected dev dependency shape");

    assert!(
        manifest.contains("reqwest.workspace = true"),
        "reqwest must use the workspace dependency declaration"
    );

    for forbidden in ["bijux-dna-engine", "bijux-dna-planner", "bijux-dna-runtime"] {
        assert!(
            !dependencies.contains(forbidden),
            "production runtime dependency `{forbidden}` must not enter bijux-dna-dev"
        );
    }
}

fn section_keys(manifest: &str, section: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut in_section = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == section;
            continue;
        }
        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, _value)) = line.split_once('=') {
            keys.insert(key.trim().split('.').next().unwrap_or_default().to_string());
        }
    }

    keys
}
