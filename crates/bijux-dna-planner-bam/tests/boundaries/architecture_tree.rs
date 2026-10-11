use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn planner_bam_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );
    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "api.rs",
            "execution_graph.rs",
            "lib.rs",
            "local_readiness.rs",
            "benchmark/",
            "benchmark.rs",
            "params.rs",
            "profile_catalog.rs",
            "report_stage.rs",
            "selection/",
            "stage_activation.rs",
            "stage_dispatch/",
            "stages/",
            "tool_adapters/",
            "tool_policy.rs",
        ]),
        "planner src tree must keep API, selection, dispatch, stages, and adapters separated"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark")),
        entries(["cases/"]),
        "benchmark planning owns governed stage cases"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark/cases")),
        entries(["admission.rs", "ancient/", "downstream/", "mod.rs", "pre/", "qc/"]),
        "stage families own case contracts"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark/cases/pre")),
        entries([
            "mod.rs",
            "validate.rs",
            "qc_pre.rs",
            "mapping_summary.rs",
            "filter.rs",
            "mapq_filter.rs",
            "length_filter.rs",
            "overlap_correction.rs"
        ]),
        "each stage owns its governed config, case inputs and plan construction"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark/cases/qc")),
        entries([
            "mod.rs",
            "markdup.rs",
            "duplication_metrics.rs",
            "complexity.rs",
            "coverage.rs",
            "insert_size.rs",
            "gc_bias.rs",
            "endogenous_content.rs",
            "recalibration.rs"
        ]),
        "each stage owns its governed config, case inputs and plan construction"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark/cases/ancient")),
        entries(["mod.rs", "damage.rs", "authenticity.rs", "sex.rs"]),
        "each stage owns its governed config, case inputs and plan construction"
    );
    assert_eq!(
        dir_entries(&root.join("src/benchmark/cases/downstream")),
        entries(["mod.rs", "bias_mitigation.rs", "kinship.rs"]),
        "each stage owns its governed config, case inputs and plan construction"
    );
    assert_eq!(
        dir_entries(&root.join("src/selection")),
        entries([
            "domain_tool_output_contracts.rs",
            "output_readiness.rs",
            "domain_tool_specs.rs",
            "mod.rs",
            "registry.rs",
            "readiness.rs",
            "tool_selection.rs",
        ]),
        "selection must separate registry loading from tool choice logic"
    );
    assert_eq!(
        dir_entries(&root.join("src/stage_dispatch")),
        entries(["adna.rs", "downstream.rs", "mod.rs", "planning.rs", "post.rs", "pre.rs"]),
        "stage dispatch must stay partitioned by BAM planning family"
    );
    assert_eq!(
        dir_entries(&root.join("src/stages")),
        entries(["mod.rs", "stage_catalog.rs"]),
        "stage registry projection must stay isolated"
    );
    assert_eq!(
        dir_entries(&root.join("src/tool_adapters")),
        entries([
            "bam.rs",
            "mod.rs",
            "stages_adna.rs",
            "stages_downstream.rs",
            "stages_post.rs",
            "stages_pre.rs",
            "stages_support.rs",
            "tools/",
        ]),
        "tool adapters must separate stage families and tool metadata"
    );
    assert_eq!(
        dir_entries(&root.join("src/tool_adapters/tools")),
        entries(["catalog.rs", "core/", "downstream/", "mod.rs", "pre/"]),
        "tool metadata must stay partitioned by BAM command group"
    );
    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "snapshots/",
        ]),
        "test tree must stay organized by durable intent"
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
