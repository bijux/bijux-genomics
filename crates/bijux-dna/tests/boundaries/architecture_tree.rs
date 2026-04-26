use std::collections::BTreeSet;

#[test]
#[allow(clippy::too_many_lines)]
fn dna_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> =
        ["BOUNDARY.md", "Cargo.toml", "PUBLIC_API.md", "README.md", "docs/", "src/", "tests/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(root_entries, expected_root, "dna crate root must stay minimal and intentional");

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> =
        ["bin/", "cli_entrypoint.rs", "commands/", "lib.rs", "process_exit.rs", "public_api/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(src_entries, expected_src, "dna src tree must match the documented CLI layout");

    let command_entries = dir_entries(&root.join("src/commands"));
    let expected_commands: BTreeSet<_> = [
        "bam/",
        "benchmark/",
        "cli/",
        "corpus/",
        "ena/",
        "example/",
        "example.rs",
        "fastq/",
        "hpc/",
        "mod.rs",
        "planning/",
        "router/",
        "status/",
        "support/",
        "vcf/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        command_entries, expected_commands,
        "dna commands tree must stay partitioned by enduring concern"
    );

    let router_entries = dir_entries(&root.join("src/commands/router"));
    let expected_router: BTreeSet<_> =
        ["argv.rs", "entrypoint.rs", "mod.rs", "root.rs", "root_commands.rs", "runtime.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        router_entries, expected_router,
        "router tree must stay focused on CLI entry and routing"
    );

    let benchmark_entries = dir_entries(&root.join("src/commands/benchmark"));
    let expected_benchmark: BTreeSet<_> = [
        "config.rs",
        "corpus_fastq/",
        "corpus_metadata.rs",
        "fastq_bench/",
        "mod.rs",
        "publication/",
        "repo_checks.rs",
        "stage_catalog.rs",
        "suite/",
        "taxonomy_database.rs",
        "workspace/",
        "workspace.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        benchmark_entries, expected_benchmark,
        "benchmark tree must keep benchmark-specific workflows together"
    );

    let benchmark_fastq_bench_entries =
        dir_entries(&root.join("src/commands/benchmark/fastq_bench"));
    let expected_benchmark_fastq_bench: BTreeSet<_> =
        ["adapter_discovery.rs", "discovery.rs", "explain.rs", "mod.rs", "tool_policy.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        benchmark_fastq_bench_entries, expected_benchmark_fastq_bench,
        "fastq benchmark tree must separate adapter discovery, stage discovery, explanation, and tool policy"
    );

    let benchmark_corpus_fastq_entries =
        dir_entries(&root.join("src/commands/benchmark/corpus_fastq"));
    let expected_benchmark_corpus_fastq: BTreeSet<_> = [
        "artifact_bundle.rs",
        "mod.rs",
        "models.rs",
        "report_qc_support.rs",
        "runtime_support.rs",
        "sortmerna_support.rs",
        "stage_preparation.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        benchmark_corpus_fastq_entries, expected_benchmark_corpus_fastq,
        "corpus fastq benchmark tree must separate run models, runtime support, stage preparation, and governed support"
    );

    let cli_entries = dir_entries(&root.join("src/commands/cli"));
    let expected_cli: BTreeSet<_> = [
        "env/",
        "execute.rs",
        "mod.rs",
        "parse/",
        "parse.rs",
        "plan/",
        "plan.rs",
        "render/",
        "validate.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        cli_entries, expected_cli,
        "cli tree must keep parse, render, plan, and environment concerns explicit"
    );

    let cli_parse_entries = dir_entries(&root.join("src/commands/cli/parse"));
    let expected_cli_parse: BTreeSet<_> = [
        "bam.rs",
        "bench/",
        "ci.rs",
        "common_example_args.rs",
        "common_root_args.rs",
        "fastq.rs",
        "parse_env_and_pipeline.rs",
        "parse_root_and_analyze.rs",
        "vcf.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        cli_parse_entries, expected_cli_parse,
        "cli parse tree must keep bench parsing separate from shared root parsing"
    );

    let bench_parse_entries = dir_entries(&root.join("src/commands/cli/parse/bench"));
    let expected_bench_parse: BTreeSet<_> =
        ["config.rs", "corpus_fastq.rs", "fastq/", "mod.rs", "publication.rs", "suite.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        bench_parse_entries, expected_bench_parse,
        "bench parse tree must keep config, publication, suite, and fastq parsing separate"
    );

    let bench_fastq_parse_entries = dir_entries(&root.join("src/commands/cli/parse/bench/fastq"));
    let expected_bench_fastq_parse: BTreeSet<_> =
        ["mod.rs", "preprocessing.rs", "quality.rs", "workflows.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        bench_fastq_parse_entries, expected_bench_fastq_parse,
        "bench fastq parse tree must keep preprocessing, quality, and workflow parsing separate"
    );

    let benchmark_workspace_entries = dir_entries(&root.join("src/commands/benchmark/workspace"));
    let expected_benchmark_workspace: BTreeSet<_> = [
        "config_loading.rs",
        "config_paths.rs",
        "config_queries.rs",
        "contracts.rs",
        "layout_normalization.rs",
        "layout_status.rs",
        "publication_contracts.rs",
        "stage_run_layout.rs",
        "value_queries.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        benchmark_workspace_entries, expected_benchmark_workspace,
        "benchmark workspace tree must keep config, publication, layout, and value concerns separate"
    );

    let support_entries = dir_entries(&root.join("src/commands/support"));
    let expected_support: BTreeSet<_> = [
        "OWNER.toml",
        "mod.rs",
        "prelude.rs",
        "report_inputs.rs",
        "run_profile.rs",
        "workspace_audit.rs",
        "workspace_root.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        support_entries, expected_support,
        "support tree must own shared command helpers only"
    );

    let fastq_entries = dir_entries(&root.join("src/commands/fastq"));
    let expected_fastq: BTreeSet<_> =
        ["api_bridge.rs", "meta/", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        fastq_entries, expected_fastq,
        "fastq tree must keep API mediation separate from meta dispatch"
    );

    let fastq_meta_entries = dir_entries(&root.join("src/commands/fastq/meta"));
    let expected_fastq_meta: BTreeSet<_> = [
        "OWNER.toml",
        "analyze.rs",
        "debug.rs",
        "entrypoint.rs",
        "environment.rs",
        "mod.rs",
        "pipelines.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        fastq_meta_entries, expected_fastq_meta,
        "fastq meta tree must stay focused on meta-command routing"
    );

    let env_entries = dir_entries(&root.join("src/commands/cli/env"));
    let expected_env: BTreeSet<_> = [
        "env_benchmark_roots.rs",
        "env_policy_linting.rs",
        "env_promotion_and_versions.rs",
        "env_registry_commands.rs",
        "env_registry_queries.rs",
        "env_runtime_support.rs",
        "mod.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        env_entries, expected_env,
        "environment command tree must keep registry, runtime, and benchmark root concerns separate"
    );

    let planning_entries = dir_entries(&root.join("src/commands/planning"));
    let expected_planning: BTreeSet<_> =
        ["OWNER.toml", "entrypoint.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        planning_entries, expected_planning,
        "planning tree must stay focused on run planning"
    );

    let status_entries = dir_entries(&root.join("src/commands/status"));
    let expected_status: BTreeSet<_> =
        ["OWNER.toml", "entrypoint.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        status_entries, expected_status,
        "status tree must stay focused on runtime status inspection"
    );

    let corpus_entries = dir_entries(&root.join("src/commands/corpus"));
    let expected_corpus: BTreeSet<_> =
        ["OWNER.toml", "entrypoint.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        corpus_entries, expected_corpus,
        "corpus tree must stay focused on curated corpus workflows"
    );

    let public_api_entries = dir_entries(&root.join("src/public_api"));
    let expected_public_api: BTreeSet<_> =
        ["cli.rs", "hpc.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(public_api_entries, expected_public_api, "public api tree must stay curated");
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
