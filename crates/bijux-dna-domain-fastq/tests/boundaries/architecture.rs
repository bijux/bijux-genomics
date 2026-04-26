use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn crate_tree_matches_domain_fastq_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(dir_entries(root), expected_root, "domain-fastq root must stay minimal");

    let expected_docs: BTreeSet<_> = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "COMMANDS.md",
        "CONTRACTS.md",
        "DEPENDENCIES.md",
        "DOMAIN_MODEL.md",
        "EFFECTS.md",
        "INDEX.md",
        "PUBLIC_API.md",
        "TESTS.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let docs_entries = dir_entries(&root.join("docs"));
    assert_eq!(docs_entries.len(), 10, "domain-fastq docs allowance is 10 Markdown files");
    assert_eq!(docs_entries, expected_docs, "domain-fastq docs spine must stay explicit");

    let expected_src: BTreeSet<_> = [
        "artifacts/",
        "banks/",
        "bench/",
        "bench_repository.rs",
        "comparison_contract/",
        "comparison_contract.rs",
        "domain_adapter.rs",
        "execution_support/",
        "id_catalog.rs",
        "integration_matrix/",
        "integration_matrix.rs",
        "invariants/",
        "lib.rs",
        "metrics/",
        "observer/",
        "observer.rs",
        "params/",
        "pipeline_contract/",
        "prelude.rs",
        "qc_contract.rs",
        "run/",
        "stage_tool_governance/",
        "stages/",
        "types/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(dir_entries(&root.join("src")), expected_src, "source tree changed");

    assert_eq!(
        dir_entries(&root.join("src/banks")),
        entries(["adapter/", "contaminant/", "mod.rs", "polyx/", "selection/", "selection.rs"]),
        "banks/ must stay split by bank family and selection concern"
    );
    for family in ["adapter", "contaminant", "polyx"] {
        assert_eq!(
            dir_entries(&root.join("src/banks").join(family)),
            entries(["mod.rs", "models.rs", "resolution.rs", "validation.rs"]),
            "{family} bank must keep models, resolution, and validation separate"
        );
    }
    assert_eq!(
        dir_entries(&root.join("src/banks/selection")),
        entries(["adapters.rs", "contaminants.rs", "polyx.rs", "warnings.rs"]),
        "bank selection must stay split by bank family"
    );

    assert_eq!(
        dir_entries(&root.join("src/params")),
        entries([
            "defaults/",
            "descriptor/",
            "edna.rs",
            "effective.rs",
            "mod.rs",
            "parsing.rs",
            "processing/",
            "quality/",
        ]),
        "params/ must keep defaults, descriptors, parsing, and typed parameter families separate"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/defaults")),
        entries(["mod.rs", "processing.rs", "profiling.rs", "quality.rs", "shared.rs"]),
        "parameter defaults must stay grouped by semantic concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/descriptor")),
        entries(["edna.rs", "mod.rs", "processing.rs", "quality.rs"]),
        "parameter descriptors must stay grouped by semantic concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/processing")),
        entries([
            "correct.rs",
            "merge.rs",
            "mod.rs",
            "preprocess.rs",
            "reference_index.rs",
            "remove_duplicates.rs",
            "umi.rs",
        ]),
        "processing parameters must stay split by transform family"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/quality")),
        entries([
            "detect_adapters.rs",
            "filter.rs",
            "mod.rs",
            "qc_post.rs",
            "screen/",
            "stats.rs",
            "trim/",
            "validate.rs",
        ]),
        "quality parameters must stay split by quality and screening concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/quality/screen")),
        entries([
            "host_depletion.rs",
            "mod.rs",
            "reference_depletion.rs",
            "rrna_depletion.rs",
            "taxonomy.rs",
        ]),
        "screen parameters must stay split by screen target"
    );
    assert_eq!(
        dir_entries(&root.join("src/params/quality/trim")),
        entries(["mod.rs", "terminal_damage.rs", "tool_profiles.rs"]),
        "trim parameters must keep terminal damage and tool-profile policy separated"
    );

    let expected_stages: BTreeSet<_> = [
        "contract.rs",
        "contract/",
        "ids.rs",
        "mod.rs",
        "ports/",
        "semantics.rs",
        "semantics/",
        "specs.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        dir_entries(&root.join("src/stages")),
        expected_stages,
        "stages/ must contain durable stage-contract ownership modules"
    );

    let expected_tests: BTreeSet<_> = [
        "benchmark_scenario_coverage.rs",
        "boundaries/",
        "boundaries.rs",
        "comparison_contract_coverage.rs",
        "contracts/",
        "contracts.rs",
        "determinism/",
        "determinism.rs",
        "fixtures/",
        "guardrails.rs",
        "semantics/",
        "semantics.rs",
        "snapshots/",
        "support/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(dir_entries(&root.join("tests")), expected_tests, "test tree changed");
}

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    collect_markdown_outside_docs(root, root, &mut offenders);

    assert!(
        offenders.is_empty(),
        "crate markdown must be root README.md or live under docs/: {}",
        offenders.join(", ")
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

fn collect_markdown_outside_docs(root: &Path, path: &Path, offenders: &mut Vec<String>) {
    for entry in
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or_else(|err| {
            panic!("strip {} from {}: {err}", root.display(), path.display())
        });
        let rel_text = rel.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            if rel_text != "docs" {
                collect_markdown_outside_docs(root, &path, offenders);
            }
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "md")
            && rel_text != "README.md"
            && !rel_text.starts_with("docs/")
        {
            offenders.push(rel_text);
        }
    }
}
