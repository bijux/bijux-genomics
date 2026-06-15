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
        dir_entries(&root.join("src/selection")),
        entries(["domain_tool_specs.rs", "mod.rs", "registry.rs", "tool_selection.rs"]),
        "selection must separate registry loading from tool choice logic"
    );
    assert_eq!(
        dir_entries(&root.join("src/stage_dispatch")),
        entries(["adna.rs", "downstream.rs", "mod.rs", "post.rs", "pre.rs"]),
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
            "guardrails.rs",
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
