use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn normal_dependency_graph_matches_boundary() {
    let manifest = read_manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let expected = entries(["anyhow", "bijux-dna-domain-vcf", "serde", "toml"]);

    assert_eq!(dependencies, expected, "normal dependencies must match docs/DEPENDENCIES.md");
}

#[test]
fn dev_dependency_graph_stays_guardrail_only() {
    let manifest = read_manifest();
    let dependencies = dependency_names(&manifest, "dev-dependencies");

    assert_eq!(dependencies, entries(["bijux-dna-policies"]));
}

#[test]
fn db_ref_rejects_downstream_workspace_dependencies() {
    let manifest = read_manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let forbidden = [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-core",
        "bijux-dna-db-ena",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-infra",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-runtime",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ];

    for dependency in forbidden {
        assert!(
            !dependencies.contains(dependency),
            "db-ref must not depend on downstream workspace crate `{dependency}`"
        );
    }
}

fn read_manifest() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
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
