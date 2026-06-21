use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn dependency_graph_matches_domain_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    let dependencies = section_keys(&manifest, "[dependencies]");
    let expected_dependencies: BTreeSet<_> = [
        "anyhow",
        "bijux-dna-core",
        "bijux-dna-infra",
        "noodles-bam",
        "noodles-sam",
        "schemars",
        "serde",
        "serde_json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(dependencies, expected_dependencies, "unexpected direct dependency shape");

    let dev_dependencies = section_keys(&manifest, "[dev-dependencies]");
    let expected_dev_dependencies: BTreeSet<_> =
        ["bijux-dna-policies", "bijux-dna-testkit", "insta", "walkdir"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(dev_dependencies, expected_dev_dependencies, "unexpected dev dependency shape");

    assert!(
        manifest.contains("bijux-dna-policies.workspace = true"),
        "bijux-dna-policies must use the workspace dependency declaration"
    );

    for forbidden in [
        "bijux-dna-dev",
        "bijux-dna-engine",
        "bijux-dna-planner-bam",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-stages-bam",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "execution/planner dependency `{forbidden}` must not enter bijux-dna-domain-bam"
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
