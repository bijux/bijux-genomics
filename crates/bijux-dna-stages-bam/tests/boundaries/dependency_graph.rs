use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn normal_dependency_graph_matches_stages_bam_boundary() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let expected = entries([
        "anyhow",
        "bijux-dna-core",
        "bijux-dna-domain-bam",
        "bijux-dna-infra",
        "bijux-dna-stage-contract",
        "serde",
        "serde_json",
    ]);

    assert_eq!(
        dependencies, expected,
        "normal dependencies must stay limited to BAM stage contracts"
    );
}

#[test]
fn dev_dependency_graph_stays_test_facing() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dev-dependencies");
    let expected = entries(["bijux-dna-policies", "bijux-dna-testkit", "walkdir"]);

    assert_eq!(dependencies, expected, "dev dependencies must stay test-facing");
}

#[test]
fn stages_bam_rejects_planner_runtime_runner_api_and_environment_edges() {
    let manifest = manifest();
    let dependencies = dependency_names(&manifest, "dependencies");
    let forbidden = [
        "bijux-dna",
        "bijux-dna-analyze",
        "bijux-dna-api",
        "bijux-dna-engine",
        "bijux-dna-environment",
        "bijux-dna-environment-qa",
        "bijux-dna-pipelines",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-vcf",
        "bijux-dna-runner",
        "bijux-dna-runtime",
    ];

    for dependency in forbidden {
        assert!(
            !dependencies.contains(dependency),
            "bijux-dna-stages-bam must not depend on `{dependency}`"
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
