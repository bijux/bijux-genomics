use std::collections::BTreeSet;
use std::path::Path;

#[test]
#[allow(clippy::too_many_lines)]
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

    assert_eq!(
        dir_entries(&root.join("src/observer")),
        entries(["contracts/", "parse/"]),
        "observer/ must stay split between contract catalog and parser implementation"
    );
    assert_eq!(
        dir_entries(&root.join("src/observer/contracts")),
        entries(["amplicon.rs", "catalog.rs", "core.rs", "mod.rs", "queries.rs", "transform.rs"]),
        "observer contracts must stay split by contract family and query surface"
    );
    assert_eq!(
        dir_entries(&root.join("src/observer/parse")),
        entries([
            "adapter_taxonomy.rs",
            "correct_errors.rs",
            "depletion/",
            "duplicates.rs",
            "filtering.rs",
            "mod.rs",
            "parser_contracts/",
            "profiles/",
            "reports.rs",
            "sequence.rs",
            "tool_metrics.rs",
        ]),
        "observer parsers must stay grouped by report family"
    );
    assert_eq!(
        dir_entries(&root.join("src/observer/parse/depletion")),
        entries(["host.rs", "mod.rs", "reference_contaminants.rs", "rrna.rs"]),
        "depletion parsers must stay split by depletion target"
    );
    assert_eq!(
        dir_entries(&root.join("src/observer/parse/profiles")),
        entries(["mod.rs", "overrepresented.rs", "read_lengths.rs", "reads.rs"]),
        "profile parsers must stay split by profile output family"
    );

    assert_eq!(
        dir_entries(&root.join("src/metrics")),
        entries(["deltas.rs", "mod.rs", "spec/", "types.rs", "types/"]),
        "metrics/ must separate specs, deltas, and value types"
    );
    assert_eq!(
        dir_entries(&root.join("src/metrics/spec")),
        entries(["catalog.rs", "classes.rs", "mod.rs"]),
        "metric specs must keep catalog and class vocabulary separate"
    );
    assert_eq!(
        dir_entries(&root.join("src/metrics/types")),
        entries([
            "classification.rs",
            "common.rs",
            "stage_metrics/",
            "summaries.rs",
            "tool_metrics.rs",
        ]),
        "metric value types must stay grouped by metric family"
    );
    assert_eq!(
        dir_entries(&root.join("src/metrics/types/stage_metrics")),
        entries(["cleanup.rs", "mod.rs", "reporting.rs", "transforms.rs", "validation.rs"]),
        "stage metric value types must stay split by stage metric family"
    );
    assert_eq!(
        dir_entries(&root.join("src/invariants")),
        entries(["edna.rs", "evaluation.rs", "metrics/", "mod.rs", "specs.rs"]),
        "invariants/ must separate specs, evaluation, eDNA, and metric evaluators"
    );
    assert_eq!(
        dir_entries(&root.join("src/invariants/metrics")),
        entries([
            "evaluate.rs",
            "merge.rs",
            "mod.rs",
            "shared.rs",
            "stage_sets.rs",
            "trim_filter.rs",
            "validation.rs",
        ]),
        "metric invariant evaluators must stay split by metric family"
    );

    assert_eq!(
        dir_entries(&root.join("src/pipeline_contract")),
        entries(["catalog.rs", "catalog/", "graph.rs", "graph/", "mod.rs"]),
        "pipeline_contract/ must keep catalog and graph concerns separate"
    );
    assert_eq!(
        dir_entries(&root.join("src/pipeline_contract/catalog")),
        entries(["criticality.rs", "modes.rs", "ordering.rs", "transitions.rs"]),
        "pipeline catalog must stay split by catalog concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/pipeline_contract/graph")),
        entries(["assembly.rs", "dependencies.rs", "edges.rs"]),
        "pipeline graph must keep assembly, dependencies, and edges separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/stage_tool_governance")),
        entries([
            "input_layout.rs",
            "layout_catalog.rs",
            "mod.rs",
            "model.rs",
            "profiles.rs",
            "readiness.rs",
        ]),
        "stage-tool governance must keep layout, profiles, readiness, and model concerns separate"
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
    assert_eq!(
        dir_entries(&root.join("src/stages/contract")),
        entries(["catalog.rs", "export.rs", "runtime/"]),
        "stage contracts must keep catalog, export, and runtime policy separated"
    );
    assert_eq!(
        dir_entries(&root.join("src/stages/contract/runtime")),
        entries([
            "header_inspection.rs",
            "merge_suitability.rs",
            "mod.rs",
            "output_normalization.rs",
        ]),
        "stage runtime contract helpers must stay split by runtime preflight concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/stages/ports")),
        entries(["manifest.rs", "mod.rs", "queries.rs"]),
        "stage ports must separate manifest data and query helpers"
    );
    assert_eq!(
        dir_entries(&root.join("src/stages/semantics")),
        entries(["catalog/", "queries.rs"]),
        "stage semantics must separate catalog data and query helpers"
    );
    assert_eq!(
        dir_entries(&root.join("src/stages/semantics/catalog")),
        entries([
            "amplicon.rs",
            "boundaries.rs",
            "cleanup.rs",
            "mod.rs",
            "screening.rs",
            "transforms.rs",
        ]),
        "stage semantics catalog must stay split by semantic stage family"
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
