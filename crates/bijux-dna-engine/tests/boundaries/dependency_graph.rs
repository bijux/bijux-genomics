use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn normal_dependency_graph_matches_engine_boundary() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let expected = entries([
        "anyhow",
        "bijux-dna-core",
        "bijux-dna-infra",
        "bijux-dna-runtime",
        "chrono",
        "serde",
        "serde_json",
        "thiserror",
        "tracing",
    ]);

    assert_eq!(
        dependencies, expected,
        "normal dependencies must stay limited to engine orchestration contracts"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dev-dependencies");
    let expected = entries(["bijux-dna-policies", "cargo_metadata", "tempfile", "walkdir"]);

    assert_eq!(dependencies, expected, "dev dependencies must stay test-facing");
}

#[test]
fn engine_rejects_planner_domain_stage_runner_and_environment_dependencies() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let forbidden = [
        "bijux-dna-api",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-vcf",
        "bijux-dna-environment",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-stage-contract",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-vcf",
    ];

    for dependency in forbidden {
        assert!(
            !dependencies.contains(dependency),
            "bijux-dna-engine must not depend on `{dependency}`"
        );
    }
}

fn manifest() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"))
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
