#[path = "../../support.rs"]
mod support;

#[test]
fn tree_contract_is_minimal() {
    let root = support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let expected = [
        "BOUNDARY.md",
        "Cargo.toml",
        "PUBLIC_API.md",
        "README.md",
        "docs/",
        "src/",
        "tests/",
    ];
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read crate root") {
        let entry = entry.expect("read entry");
        let path = entry.path();
        let name = if path.is_dir() {
            format!("{}/", entry.file_name().to_string_lossy())
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        entries.push(name);
    }
    entries.sort();
    let expected_set: std::collections::BTreeSet<_> =
        expected.iter().map(|s| s.to_string()).collect();
    let entries_set: std::collections::BTreeSet<_> = entries.into_iter().collect();
    assert_eq!(
        entries_set, expected_set,
        "Stage-contract tree must remain minimal; update tree contract intentionally."
    );

    let src_dir = root.join("src");
    let allowed_src = [
        "execution_plan.rs",
        "execution_plan_support.rs",
        "execution_plan_validation.rs",
        "execution_step.rs",
        "executor_registry.rs",
        "executor_registry_catalog.rs",
        "executor_registry_lookup.rs",
        "lib.rs",
        "plan_edge.rs",
        "plan_run.rs",
        "planner_contract.rs",
        "run_artifact_catalog.rs",
        "run_execution_builder.rs",
        "stage_plan.rs",
        "stage_plan_json.rs",
        "stage_plugin.rs",
        "stage_reason.rs",
    ];
    let mut src_entries = Vec::new();
    for entry in std::fs::read_dir(&src_dir).expect("read src dir") {
        let entry = entry.expect("read src entry");
        let name = entry.file_name().to_string_lossy().to_string();
        src_entries.push(name);
    }
    src_entries.sort();
    let allowed_set: std::collections::BTreeSet<_> =
        allowed_src.iter().map(|s| s.to_string()).collect();
    let src_set: std::collections::BTreeSet<_> = src_entries.into_iter().collect();
    assert_eq!(
        src_set, allowed_set,
        "Stage-contract src must match the contract architecture layout."
    );
}
