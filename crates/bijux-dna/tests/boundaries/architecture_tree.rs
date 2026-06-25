use std::collections::BTreeSet;

#[test]
#[allow(clippy::too_many_lines)]
fn dna_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(root_entries, expected_root, "dna crate root must stay minimal and intentional");

    let docs_entries = dir_entries(&root.join("docs"));
    let expected_docs: BTreeSet<_> = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "CHANGE_RULES.md",
        "COMMANDS.md",
        "DRY_RUN.md",
        "EFFECTS.md",
        "INDEX.md",
        "OUTPUT_FORMATS.md",
        "PUBLIC_API.md",
        "TESTS.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(docs_entries, expected_docs, "dna docs must stay capped at ten files");

    let markdown_entries = markdown_entries(&root);
    let expected_markdown: BTreeSet<_> = std::iter::once("README.md".to_string())
        .chain(expected_docs.iter().map(|file| format!("docs/{file}")))
        .collect();
    assert_eq!(
        markdown_entries, expected_markdown,
        "dna markdown must be limited to root README.md and docs/*.md"
    );

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
        "ci.rs",
        "cli/",
        "corpus/",
        "crates.rs",
        "ena/",
        "example/",
        "example.rs",
        "fastq/",
        "fixtures/",
        "hpc/",
        "mod.rs",
        "numeric.rs",
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

    let fixture_entries = dir_entries(&root.join("src/commands/fixtures"));
    let expected_fixtures: BTreeSet<_> =
        ["build/", "entrypoint.rs", "expected/", "mod.rs", "paths.rs", "root_validation.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        fixture_entries, expected_fixtures,
        "fixture commands must keep generation, expected-truth, entrypoint, and path ownership separate"
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
        "active_scope.rs",
        "alias_inventory.rs",
        "bam_stage_families.rs",
        "benchmark_result_ids.rs",
        "config.rs",
        "corpus_fastq/",
        "corpus_metadata.rs",
        "fastq_bench/",
        "fastq_stage_families.rs",
        "local_adna_micro_pipeline.rs",
        "local_all_domain_fake_failures.rs",
        "local_all_domain_fake_runs.rs",
        "local_all_domain_job_execution.rs",
        "local_all_domain_result_paths.rs",
        "local_all_domain_slurm_path_convention.rs",
        "local_all_domain_slurm_script_bodies.rs",
        "local_all_domain_slurm_scripts.rs",
        "local_all_domain_slurm_shell_syntax.rs",
        "local_all_domain_slurm_submit_manifest.rs",
        "local_amplicon_micro_pipeline.rs",
        "local_bam_micro_smoke_subset.rs",
        "local_bam_stage_smoke.rs",
        "local_benchmark_summary.rs",
        "local_core_germline_micro_pipeline.rs",
        "local_corpus_fixture/",
        "local_corpus_skip_report.rs",
        "local_corpus_stage_compatibility.rs",
        "local_cross_domain_sample_consistency.rs",
        "local_dag_watchdog_simulation.rs",
        "local_edna_micro_pipeline.rs",
        "local_essential_pipeline_fake_runs.rs",
        "local_fastq_micro_smoke_subset.rs",
        "local_hpc_array_support.rs",
        "local_hpc_asset_staging_manifest.rs",
        "local_hpc_candidate_run_manifest.rs",
        "local_hpc_dependency_simulation.rs",
        "local_hpc_dry_run_ready.rs",
        "local_hpc_execution_resolver.rs",
        "local_hpc_input_discovery.rs",
        "local_hpc_job_completion.rs",
        "local_hpc_job_graph.rs",
        "local_hpc_job_resources.rs",
        "local_hpc_pipeline_node_array.rs",
        "local_hpc_result_collection_simulation.rs",
        "local_hpc_resume_simulation.rs",
        "local_hpc_scratch_layout.rs",
        "local_hpc_selected_jobs.rs",
        "local_hpc_simulation_tree.rs",
        "local_hpc_stage_benchmark_array.rs",
        "local_hpc_submission_ready.rs",
        "local_micro_benchmark_report.rs",
        "local_micro_benchmark_run.rs",
        "local_pipeline_dag.rs",
        "local_real_smoke_core_subset.rs",
        "local_slurm_dependency_check.rs",
        "local_slurm_dry_run.rs",
        "local_slurm_run_paths.rs",
        "local_slurm_script_bodies.rs",
        "local_slurm_shell_syntax.rs",
        "local_slurm_submit_manifest.rs",
        "local_stage_commands.rs",
        "local_stage_fake_runs.rs",
        "local_stage_inventory.rs",
        "local_stage_manifest_completion.rs",
        "local_stage_output_completion.rs",
        "local_stage_result_manifest.rs",
        "local_stage_runtime_metrics.rs",
        "local_taxonomy_database_fixture.rs",
        "local_taxonomy_output_judgment.rs",
        "local_tool_comparison_template.rs",
        "local_vcf_admixture_smoke.rs",
        "local_vcf_call_bam_smoke_support.rs",
        "local_vcf_call_diploid_smoke.rs",
        "local_vcf_call_gl_smoke.rs",
        "local_vcf_call_pseudohaploid_smoke.rs",
        "local_vcf_call_smoke.rs",
        "local_vcf_damage_filter_smoke.rs",
        "local_vcf_demography_smoke.rs",
        "local_vcf_filter_smoke.rs",
        "local_vcf_gl_propagation_smoke.rs",
        "local_vcf_ibd_smoke.rs",
        "local_vcf_imputation_metrics_smoke.rs",
        "local_vcf_impute_smoke.rs",
        "local_vcf_micro_smoke_subset.rs",
        "local_vcf_no_empty_output.rs",
        "local_vcf_panel_workflow_smoke_support.rs",
        "local_vcf_pca_smoke.rs",
        "local_vcf_phasing_smoke.rs",
        "local_vcf_population_structure_smoke.rs",
        "local_vcf_postprocess_smoke.rs",
        "local_vcf_prepare_reference_panel_smoke.rs",
        "local_vcf_qc_smoke.rs",
        "local_vcf_reference_compatibility.rs",
        "local_vcf_roh_smoke.rs",
        "local_vcf_sample_compatibility.rs",
        "local_vcf_smoke_root.rs",
        "local_vcf_smoke_suite_ready.rs",
        "local_vcf_stage_catalog.rs",
        "local_vcf_stage_catalog_ready.rs",
        "local_vcf_stage_matrix.rs",
        "local_vcf_stats_smoke.rs",
        "mod.rs",
        "path_resolution.rs",
        "paths.rs",
        "publication/",
        "readiness/",
        "repo_checks.rs",
        "schema_paths.rs",
        "schema_validation.rs",
        "stage_catalog.rs",
        "suite/",
        "taxonomy_database.rs",
        "vcf_benchmark_bindings.rs",
        "vcf_stage_families.rs",
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
        "dev.rs",
        "fastq.rs",
        "fixtures.rs",
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
    let expected_bench_parse: BTreeSet<_> = [
        "active_scope.rs",
        "config.rs",
        "corpus_fastq.rs",
        "fastq/",
        "local.rs",
        "matrix.rs",
        "micro.rs",
        "mod.rs",
        "paths.rs",
        "publication.rs",
        "readiness.rs",
        "schema_validation.rs",
        "suite.rs",
    ]
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
        ["api_translation.rs", "meta/", "mod.rs"].into_iter().map(str::to_string).collect();
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

    let tests_entries = dir_entries(&root.join("tests"));
    let required_test_entries: BTreeSet<_> = [
        "boundaries/",
        "boundaries.rs",
        "contracts/",
        "contracts.rs",
        "guardrails.rs",
        "schemas/",
        "schemas.rs",
        "snapshots/",
        "snapshots.rs",
        "support/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert!(
        required_test_entries.is_subset(&tests_entries),
        "dna tests tree must retain the governed grouped suite roots"
    );
    let allowed_root_test_files =
        ["boundaries.rs", "contracts.rs", "guardrails.rs", "schemas.rs", "snapshots.rs"];
    let allowed_root_test_prefixes = ["bench_", "ci_", "config_", "dev_", "fixtures_", "plan_"];
    for entry in &tests_entries {
        if required_test_entries.contains(entry) {
            continue;
        }
        assert!(
            !entry.ends_with('/'),
            "dna tests tree must not add unmanaged suite directories: {entry}"
        );
        let allowed_exact = allowed_root_test_files.iter().any(|allowed| entry == allowed);
        let allowed_prefix =
            allowed_root_test_prefixes.iter().any(|prefix| entry.starts_with(prefix));
        assert!(
            allowed_exact || allowed_prefix,
            "dna tests tree must use grouped suite roots or governed integration-test prefixes, found `{entry}`"
        );
    }

    let support_test_entries = dir_entries(&root.join("tests/support"));
    let expected_support_tests: BTreeSet<_> =
        ["workspace_paths.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        support_test_entries, expected_support_tests,
        "dna test support must keep shared helpers out of suite roots"
    );

    let boundary_test_entries = dir_entries(&root.join("tests/boundaries"));
    let expected_boundary_tests: BTreeSet<_> = [
        "architecture_tree.rs",
        "command_inventory.rs",
        "dependency_graph.rs",
        "docs_layout.rs",
        "guardrails/",
        "guardrails.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        boundary_test_entries, expected_boundary_tests,
        "dna boundary tests must stay focused on architecture, docs, dependencies, and guardrails"
    );

    let schema_test_entries = dir_entries(&root.join("tests/schemas"));
    let expected_schema_tests: BTreeSet<_> =
        ["public_surface.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        schema_test_entries, expected_schema_tests,
        "dna schema tests must own public-surface snapshot locks"
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

fn markdown_entries(root: &std::path::Path) -> BTreeSet<String> {
    fn visit(root: &std::path::Path, dir: &std::path::Path, entries: &mut BTreeSet<String>) {
        for entry in
            std::fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()))
        {
            let entry =
                entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", dir.display()));
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, entries);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or_else(|err| panic!("strip {}: {err}", path.display()))
                    .to_string_lossy()
                    .replace('\\', "/");
                entries.insert(rel);
            }
        }
    }

    let mut entries = BTreeSet::new();
    visit(root, root, &mut entries);
    entries
}
