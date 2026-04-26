use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn planner_vcf_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        child_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root should expose only the manifest, root README, docs, src, and tests"
    );
    assert_eq!(
        child_entries(&root.join("src")),
        entries([
            "api.rs",
            "chunk_plan.rs",
            "coverage.rs",
            "execution_graph.rs",
            "explain.rs",
            "explain_model.rs",
            "input_policy.rs",
            "lib.rs",
            "params.rs",
            "planner.rs",
            "reference_context.rs",
            "stage_io.rs",
            "stage_plan.rs",
            "stage_sequence.rs",
            "tool_catalog.rs",
            "tool_selection.rs",
            "workspace_config.rs",
        ]),
        "src layout should keep planner responsibilities explicit"
    );
    assert_eq!(
        child_entries(&root.join("tests")),
        entries(["boundaries/", "boundaries.rs", "contracts.rs", "guardrails.rs", "snapshots/"]),
        "test layout should keep boundary, contract, guardrail, and snapshot coverage visible"
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
