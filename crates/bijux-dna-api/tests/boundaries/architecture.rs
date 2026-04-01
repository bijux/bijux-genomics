use std::collections::BTreeSet;

#[test]
fn api_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = [
        "BOUNDARY.md",
        "Cargo.toml",
        "PUBLIC_API.md",
        "README.md",
        "docs/",
        "src/",
        "tests/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        root_entries, expected_root,
        "api crate root must stay minimal and intentional"
    );

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> = [
        "internal/",
        "lib.rs",
        "runtime/",
        "support/",
        "surface/",
        "v1/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        src_entries, expected_src,
        "api src tree must match the documented architecture"
    );

    let surface_entries = dir_entries(&root.join("src/surface"));
    let expected_surface: BTreeSet<_> = ["explain.rs", "mod.rs", "request_contracts.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        surface_entries, expected_surface,
        "api surface tree must stay focused on stable contracts"
    );

    let runtime_entries = dir_entries(&root.join("src/runtime"));
    let expected_runtime: BTreeSet<_> = [
        "cross_runtime.rs",
        "execution_kernel.rs",
        "invocation_policy/",
        "invocation_policy.rs",
        "mod.rs",
        "persistence.rs",
        "run/",
        "validation.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        runtime_entries, expected_runtime,
        "api runtime tree must stay decomposed by enduring concern"
    );

    let invocation_policy_entries = dir_entries(&root.join("src/runtime/invocation_policy"));
    let expected_invocation_policy: BTreeSet<_> =
        ["config.rs", "contracts.rs", "models.rs", "resilience.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        invocation_policy_entries, expected_invocation_policy,
        "api invocation policy support tree must stay explicit"
    );

    let support_entries = dir_entries(&root.join("src/support"));
    let expected_support: BTreeSet<_> = [
        "benchmark_runtime.rs",
        "mod.rs",
        "qa.rs",
        "reference_resolution.rs",
        "tool_selection.rs",
        "tooling.rs",
        "workspace/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        support_entries, expected_support,
        "api support tree must stay partitioned by concern"
    );

    let workspace_support_entries = dir_entries(&root.join("src/support/workspace"));
    let expected_workspace_support: BTreeSet<_> = ["mod.rs", "registry.rs", "repo_root.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        workspace_support_entries, expected_workspace_support,
        "api workspace support tree must isolate repository-scoped asset resolution"
    );

    let v1_entries = dir_entries(&root.join("src/v1"));
    let expected_v1: BTreeSet<_> = [
        "api.rs",
        "bam/",
        "bench.rs",
        "env.rs",
        "fastq.rs",
        "mod.rs",
        "pipelines.rs",
        "plan.rs",
        "report/",
        "run/",
        "shared.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(v1_entries, expected_v1, "api v1 tree must stay curated");

    let v1_run_entries = dir_entries(&root.join("src/v1/run"));
    let expected_v1_run: BTreeSet<_> = ["mod.rs", "operator_failure.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        v1_run_entries, expected_v1_run,
        "api v1 run tree must separate failure contracts from runtime entrypoints"
    );

    let v1_report_entries = dir_entries(&root.join("src/v1/report"));
    let expected_v1_report: BTreeSet<_> = ["html_bundle.rs", "mod.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        v1_report_entries, expected_v1_report,
        "api v1 report tree must separate html rendering from report entrypoints"
    );
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
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
