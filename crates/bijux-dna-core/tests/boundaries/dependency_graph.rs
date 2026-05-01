use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn normal_dependency_graph_stays_low_level() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = dependency_names(&manifest, "dependencies");
    let expected =
        entries(["chrono", "regex", "serde", "serde_json", "sha2", "thiserror", "walkdir"]);

    assert_eq!(
        dependencies, expected,
        "normal dependencies must stay low-level and match docs/ARCHITECTURE.md"
    );
}

#[test]
fn dev_dependency_graph_stays_test_only() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = dependency_names(&manifest, "dev-dependencies");
    let expected = entries(["anyhow", "bijux-dna-policies", "insta", "tempfile"]);

    assert_eq!(dependencies, expected, "dev dependencies must stay test-facing");
}

#[test]
fn core_manifest_rejects_downstream_crate_dependencies() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = dependency_names(&manifest, "dependencies");
    let forbidden = [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-analyze",
        "bijux-dna-bench",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-infra",
        "bijux-dna-pipelines",
    ];

    for name in forbidden {
        assert!(
            !dependencies.contains(name),
            "bijux-dna-core must not depend on downstream crate `{name}`"
        );
    }
}

#[test]
fn normal_dependencies_reject_workspace_crates() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    let dependencies = dependency_names(&manifest, "dependencies");
    let workspace_dependencies = dependencies
        .iter()
        .filter(|dependency| dependency.starts_with("bijux-"))
        .collect::<Vec<_>>();

    assert!(
        workspace_dependencies.is_empty(),
        "bijux-dna-core normal dependencies must not include workspace crates: {workspace_dependencies:?}"
    );
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
