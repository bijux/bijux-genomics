use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn planner_fastq_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        child_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay narrow and reviewable"
    );
    assert_eq!(
        child_entries(&root.join("src")),
        entries([
            "compose/",
            "lib.rs",
            "pipeline_defaults.rs",
            "planner/",
            "preprocess/",
            "qc_contract.rs",
            "report_stage.rs",
            "selection/",
            "stage_api.rs",
            "surface.rs",
            "tool_adapters/",
            "tool_policy.rs",
            "unit_checks.rs",
        ]),
        "src layout must keep planner concerns separated"
    );
    assert_eq!(
        child_entries(&root.join("src/compose")),
        entries([
            "input_resolution.rs",
            "lineage.rs",
            "mod.rs",
            "models.rs",
            "qc_inputs.rs",
            "stage_params.rs",
        ]),
        "compose/ must keep routing and stage parameter concerns separated"
    );
    assert_eq!(
        child_entries(&root.join("src/planner")),
        entries([
            "benchmark.rs",
            "graph_policy.rs",
            "layout_branching.rs",
            "local_readiness.rs",
            "mod.rs",
            "quality_sampling.rs",
            "route_expansion.rs",
            "selection_planning.rs",
            "types.rs",
        ]),
        "planner/ must stay split by graph concern"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages")),
        entries(["amplicon/", "catalog.rs", "mod.rs", "pre/", "qc/", "transform/"]),
        "stage adapters must stay grouped by stage family"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages/pre")),
        entries([
            "detect_adapters.rs",
            "index_reference.rs",
            "mod.rs",
            "plan_preprocess.rs",
            "preprocess.rs",
            "profile_overrepresented_sequences.rs",
            "profile_read_lengths.rs",
            "validate_reads.rs",
        ]),
        "pre adapters must stay focused on validation, profiling, adapter detection, indexing, and preprocess planning"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages/qc")),
        entries([
            "deplete_rrna.rs",
            "mod.rs",
            "profile_reads.rs",
            "report_qc.rs",
            "screen_taxonomy.rs"
        ]),
        "qc adapters must stay focused on reporting and screening stages"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages/transform")),
        entries([
            "correct_errors.rs",
            "deplete_host.rs",
            "deplete_reference_contaminants.rs",
            "extract_umis.rs",
            "filter_low_complexity.rs",
            "filter_reads.rs",
            "merge_pairs.rs",
            "mod.rs",
            "remove_duplicates.rs",
            "trim_polyg_tails.rs",
            "trim_reads/",
            "trim_terminal_damage.rs",
        ]),
        "transform adapters must stay split by mutating FASTQ stage"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages/transform/trim_reads")),
        entries(["config.rs", "mod.rs", "reporting.rs"]),
        "trim_reads adapter must keep config and report construction out of the stage planner"
    );
    assert_eq!(
        child_entries(&root.join("src/tool_adapters/stages/amplicon")),
        entries([
            "cluster_otus.rs",
            "infer_asvs.rs",
            "mod.rs",
            "normalize_abundance.rs",
            "normalize_primers.rs",
            "remove_chimeras.rs",
        ]),
        "amplicon adapters must stay split by amplicon stage"
    );
    assert_eq!(
        child_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "snapshots/",
            "support/",
        ]),
        "tests layout must stay grouped by contract type"
    );
}

fn child_entries(path: &Path) -> BTreeSet<String> {
    fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| {
            let entry =
                entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() {
                format!("{file_name}/")
            } else {
                file_name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
