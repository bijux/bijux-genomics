use std::collections::BTreeSet;

#[test]
fn api_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> =
        ["BOUNDARY.md", "Cargo.toml", "PUBLIC_API.md", "README.md", "docs/", "src/", "tests/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(root_entries, expected_root, "api crate root must stay minimal and intentional");

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> =
        ["internal/", "lib.rs", "runtime/", "support/", "surface/", "v1/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(src_entries, expected_src, "api src tree must match the documented architecture");

    let surface_entries = dir_entries(&root.join("src/surface"));
    let expected_surface: BTreeSet<_> =
        ["explain.rs", "mod.rs", "request_contracts.rs"].into_iter().map(str::to_string).collect();
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
        "qa/",
        "reference_resolution/",
        "tool_selection.rs",
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
    let expected_workspace_support: BTreeSet<_> =
        ["mod.rs", "registry.rs", "repo_root.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        workspace_support_entries, expected_workspace_support,
        "api workspace support tree must isolate repository-scoped asset resolution"
    );

    let v1_entries = dir_entries(&root.join("src/v1"));
    let expected_v1: BTreeSet<_> = [
        "api/",
        "bam/",
        "bench/",
        "env/",
        "fastq/",
        "mod.rs",
        "pipelines/",
        "plan.rs",
        "report/",
        "run/",
        "shared.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(v1_entries, expected_v1, "api v1 tree must stay curated");

    let runtime_run_entries = dir_entries(&root.join("src/runtime/run"));
    let expected_runtime_run: BTreeSet<_> =
        ["execution/", "execution_support.rs", "mod.rs", "planning/", "reporting/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        runtime_run_entries, expected_runtime_run,
        "api runtime run tree must separate execution, planning, and reporting"
    );

    let runtime_run_planning_entries = dir_entries(&root.join("src/runtime/run/planning"));
    let expected_runtime_run_planning: BTreeSet<_> =
        ["mod.rs", "planning_support.rs", "profile_selection.rs", "run_bootstrap.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        runtime_run_planning_entries, expected_runtime_run_planning,
        "api runtime planning tree must separate selection, bootstrap, and planning support"
    );

    let runtime_run_execution_entries = dir_entries(&root.join("src/runtime/run/execution"));
    let expected_runtime_run_execution: BTreeSet<_> =
        ["mod.rs", "stage_execution.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        runtime_run_execution_entries, expected_runtime_run_execution,
        "api runtime execution tree must keep the execution entry explicit"
    );

    let v1_api_entries = dir_entries(&root.join("src/v1/api"));
    let expected_v1_api: BTreeSet<_> =
        ["front_door.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        v1_api_entries, expected_v1_api,
        "api v1 front door must stay isolated in its own namespace"
    );

    let v1_bench_entries = dir_entries(&root.join("src/v1/bench"));
    let expected_v1_bench: BTreeSet<_> =
        ["exports.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        v1_bench_entries, expected_v1_bench,
        "api v1 benchmark tree must stay isolated in its own namespace"
    );

    let v1_env_entries = dir_entries(&root.join("src/v1/env"));
    let expected_v1_env: BTreeSet<_> =
        ["mod.rs", "runtime.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        v1_env_entries, expected_v1_env,
        "api v1 environment tree must stay isolated in its own namespace"
    );

    let v1_fastq_entries = dir_entries(&root.join("src/v1/fastq"));
    let expected_v1_fastq: BTreeSet<_> =
        ["domain.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        v1_fastq_entries, expected_v1_fastq,
        "api v1 fastq tree must stay isolated in its own namespace"
    );

    let v1_pipelines_entries = dir_entries(&root.join("src/v1/pipelines"));
    let expected_v1_pipelines: BTreeSet<_> =
        ["mod.rs", "registry.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        v1_pipelines_entries, expected_v1_pipelines,
        "api v1 pipelines tree must stay isolated in its own namespace"
    );

    let v1_run_entries = dir_entries(&root.join("src/v1/run"));
    let expected_v1_run: BTreeSet<_> = [
        "entrypoints.rs",
        "mod.rs",
        "operator_failure.rs",
        "request_contracts.rs",
        "runtime_support.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        v1_run_entries, expected_v1_run,
        "api v1 run tree must separate failure contracts from runtime entrypoints"
    );

    let v1_report_entries = dir_entries(&root.join("src/v1/report"));
    let expected_v1_report: BTreeSet<_> =
        ["analysis_exports.rs", "html_bundle.rs", "mod.rs", "request_contracts.rs"]
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
        .filter(|entry| entry.file_name().to_string_lossy() != ".DS_Store")
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
