use std::fs;

#[test]
fn source_modules_have_owner() {
    let src_dir = crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"))
        .join("src");
    let mut modules = Vec::new();
    collect_rs_files(&src_dir, &mut modules);

    let mut offenders = Vec::new();
    for module in modules {
        let Ok(contents) = fs::read_to_string(&module) else {
            continue;
        };
        let mut has_owner = false;
        for line in contents.lines().take(8) {
            if line.trim().starts_with("//!") && line.contains("Owner:") {
                has_owner = true;
                break;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }
        if !has_owner {
            offenders.push(module.display().to_string());
        }
    }

    assert!(offenders.is_empty(), "missing module owner doc comments: {offenders:?}");
}

#[test]
fn thin_cli_delegates_stage_result_admission_to_benchmark_owners() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_result_manifest.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(
        contents.contains("validate_benchmark_stage_result_path"),
        "thin CLI must delegate file-backed admission to bijux-dna-bench"
    );
    assert!(
        contents.contains("validate_benchmark_stage_result_manifest"),
        "thin CLI must delegate manifest validation to bijux-dna-bench-model through the API"
    );
    assert!(
        !contents.contains("struct BenchStageResultValidationReport"),
        "thin CLI must not own the persisted validation report"
    );
    assert!(
        !contents.contains("bijux.bench.stage_result_validation.v1"),
        "thin CLI must not own the validation schema identity"
    );
}

#[test]
fn thin_cli_delegates_dossier_index_publication_to_benchmark_owner() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/publication/dossier_index.rs");
    let cli = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));
    for service in [
        "build_benchmark_dossier_stage_entry",
        "publish_benchmark_dossier_index",
        "benchmark_dossier_path",
    ] {
        assert!(cli.contains(service), "thin CLI must invoke {service}");
    }
    for forbidden in [
        "fs::read",
        "fs::write",
        "serde_json::from_",
        "validate_benchmark_dossier_index",
        "fn render_dossier_index_markdown",
        "fn classify_run_root_source",
    ] {
        assert!(!cli.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }

    let owner_source = repo_root.join("crates/bijux-dna-bench/src/workflow/dossier_index.rs");
    let owner = fs::read_to_string(&owner_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", owner_source.display()));
    for service in [
        "pub fn build_benchmark_dossier_stage_entry",
        "pub fn publish_benchmark_dossier_index",
        "pub fn render_benchmark_dossier_index_markdown",
        "bijux_dna_infra::ensure_dir",
        "bijux_dna_infra::atomic_write_bytes",
    ] {
        assert!(owner.contains(service), "benchmark owner is missing {service}");
    }
    assert!(
        !owner.contains("fs::create_dir_all(&request.docs_root)"),
        "benchmark publication must use the shared filesystem authority"
    );
}

#[test]
fn thin_cli_delegates_workspace_stage_run_paths_to_benchmark_owner() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/workspace/stage_run_layout.rs");
    let cli = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    for service in ["benchmark_runtime_corpus_dir_name", "benchmark_stage_run_relative_root"] {
        assert!(cli.contains(service), "thin CLI must delegate to {service}");
        assert!(!cli.contains(&format!("fn {service}")), "thin CLI must not implement {service}");
    }
    for forbidden in [
        ".remote_results_template",
        ".local_cache_results_template",
        ".local_archive_results_template",
        ".corpus_root",
        ".replace(\"{corpus_id}\"",
    ] {
        assert!(!cli.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }

    let owner_source = repo_root.join("crates/bijux-dna-bench/src/workflow/benchmark_config.rs");
    let owner = fs::read_to_string(&owner_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", owner_source.display()));
    for service in
        ["pub fn benchmark_runtime_corpus_dir_name", "pub fn benchmark_stage_run_relative_root"]
    {
        assert!(owner.contains(service), "benchmark owner is missing {service}");
    }

    let api_source = repo_root.join("crates/bijux-dna-api/src/v1/bench/exports.rs");
    let api = fs::read_to_string(&api_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", api_source.display()));
    for service in ["benchmark_runtime_corpus_dir_name", "benchmark_stage_run_relative_root"] {
        assert!(api.contains(service), "stable API is missing {service}");
    }
}

#[test]
fn thin_cli_delegates_corpus_skip_publication_to_benchmark_owner() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_corpus_skip_report.rs");
    let cli = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(cli.contains("publish_benchmark_local_corpus_skip_report"));
    for forbidden in [
        "fn build_skip_report",
        "BenchmarkLocalCorpusSkip {",
        "BenchmarkLocalAssetBackedStage {",
        "BenchmarkLocalPlannerOnlyStage {",
        "std::fs::create_dir_all",
        "atomic_write_json",
    ] {
        assert!(!cli.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }

    let owner_source = repo_root.join("crates/bijux-dna-bench/src/workflow/local_corpus_skip.rs");
    let owner = fs::read_to_string(&owner_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", owner_source.display()));
    for service in [
        "pub fn derive_benchmark_local_corpus_skip_report",
        "pub fn publish_benchmark_local_corpus_skip_report",
        "validate_benchmark_local_corpus_skip_report",
        "bijux_dna_infra::atomic_write_json",
    ] {
        assert!(owner.contains(service), "benchmark owner is missing {service}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_runtime_metrics_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_runtime_metrics.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_runtime_metrics"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricEntry"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricsReport"));
    assert!(!contents.contains("bijux.bench.local_runtime_metrics.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_benchmark_summary_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_benchmark_summary.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_summary"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryStage"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryReport"));
    assert!(!contents.contains("bijux.bench.local_benchmark_summary.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_smoke_suite_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_smoke_suite_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_vcf_smoke_suite"));
    assert!(!contents.contains("struct LocalVcfSmokeSuiteGoalCheck"));
    assert!(!contents.contains("struct LocalVcfSmokeSuiteReadyReport"));
    assert!(!contents.contains("bijux.bench.local_vcf_smoke_suite_ready.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_imputation_metrics_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_vcf_imputation_metrics_smoke.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_vcf_imputation_metrics_smoke_report"));
    assert!(!contents.contains("struct QualityFieldAvailability"));
    assert!(!contents.contains("struct LocalVcfImputationMetricsSmokeReport"));
    assert!(!contents.contains("bijux.bench.local_vcf_imputation_metrics_smoke.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_ibd_smoke_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_ibd_smoke.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_vcf_ibd_smoke_report"));
    assert!(!contents.contains("struct LocalVcfIbdSmokeRow"));
    assert!(!contents.contains("struct LocalVcfIbdInsufficientOverlapProbe"));
    assert!(!contents.contains("struct LocalVcfIbdSmokeReport"));
    assert!(!contents.contains("bijux.bench.local_vcf_ibd_smoke.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_call_smoke_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_call_smoke.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_vcf_call_smoke_metrics"));
    assert!(contents.contains("validate_benchmark_local_vcf_call_smoke_report"));
    assert!(!contents.contains("struct LocalVcfCallSmokeMetrics"));
    assert!(!contents.contains("struct LocalVcfCallSmokeReport"));
    assert!(!contents.contains("bijux.bench.local_vcf_call_smoke.metrics.v1"));
    assert!(!contents.contains("bijux.bench.local_vcf_call_smoke.v1"));
}

#[test]
fn thin_cli_uses_model_owned_all_domain_parser_collector_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/all_domain_parser_collector.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_all_domain_parser_collector"));
    assert!(!contents.contains("struct AllDomainParserCollectorRow"));
    assert!(!contents.contains("struct AllDomainParserCollectorReport"));
    assert!(!contents.contains("bijux.bench.readiness.all_domain_parser_collector.v1"));
}

#[test]
fn thin_cli_uses_model_owned_all_domain_failure_classification_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root.join(
        "crates/bijux-dna/src/commands/benchmark/readiness/all_domain_failure_classification.rs",
    );
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_all_domain_failure_classification"));
    assert!(!contents.contains("struct AllDomainFailureClassificationRow {"));
    assert!(!contents.contains("struct AllDomainFailureClassificationReport {"));
    assert!(!contents.contains("\"bijux.bench.readiness.all_domain_failure_classification.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_full_benchmark_report_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/full_benchmark_report.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_full_benchmark_report"));
    assert!(!contents.contains("struct FullBenchmarkReport {"));
    assert!(!contents.contains("enum FullBenchmarkReportRowStatus {"));
    assert!(!contents.contains("ensure_full_benchmark_report_contract"));
    assert!(!contents.contains("\"bijux.bench.readiness.full_benchmark_report.v1\""));
    for projection in [
        "fn build_stage_centric_rows",
        "fn build_tool_centric_rows",
        "fn build_corpus_centric_rows",
        "fn build_pipeline_centric_rows",
        "fn build_runtime_rows",
        "fn build_memory_rows",
        "fn render_full_benchmark_report_markdown",
        "fn build_failures_section",
        "fn build_missing_result_rows",
        "fn build_unsupported_pair_rows",
        "fn collect_full_benchmark_comparable_metrics",
    ] {
        assert!(!contents.contains(projection), "thin CLI reintroduced {projection}");
    }

    let workflow_source =
        repo_root.join("crates/bijux-dna-bench/src/workflow/full_benchmark_report.rs");
    let workflow = fs::read_to_string(&workflow_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", workflow_source.display()));
    for projection in [
        "build_full_benchmark_stage_rows",
        "build_full_benchmark_tool_rows",
        "build_full_benchmark_corpus_rows",
        "build_full_benchmark_pipeline_rows",
        "build_full_benchmark_runtime_rows",
        "build_full_benchmark_memory_rows",
        "render_full_benchmark_report_markdown",
        "build_full_benchmark_failures_section",
        "build_full_benchmark_missing_result_rows",
        "build_full_benchmark_unsupported_pair_rows",
        "build_full_benchmark_comparable_metric_rows",
    ] {
        assert!(workflow.contains(projection), "benchmark owner is missing {projection}");
    }
}

#[test]
fn thin_cli_uses_model_owned_full_benchmark_result_collector_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root.join(
        "crates/bijux-dna/src/commands/benchmark/readiness/full_benchmark_result_collector.rs",
    );
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_full_benchmark_result_collector"));
    assert!(!contents.contains("struct FullBenchmarkResultCollectorRow {"));
    assert!(!contents.contains("struct FullBenchmarkResultCollectorReport {"));
    assert!(!contents.contains("enum FullBenchmarkResultSurfaceKind {"));
    assert!(!contents.contains("enum FullBenchmarkResultStatus {"));
    assert!(!contents.contains("ensure_full_benchmark_result_collector_contract"));
    assert!(!contents.contains("\"bijux.bench.readiness.full_benchmark_result_collector.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_essential_pipeline_readiness_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/essential_pipelines_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_essential_pipeline_readiness"));
    assert!(!contents.contains("struct EssentialPipelinesReadyGoalCheck"));
    assert!(!contents.contains("struct EssentialPipelinesReadyReport"));
    assert!(!contents.contains("bijux.bench.readiness.essential_pipelines_ready.v1"));
}

#[test]
fn thin_cli_uses_model_owned_essential_pipeline_fake_run_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_essential_pipeline_fake_runs.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    for validator in [
        "validate_benchmark_local_essential_pipeline_fake_run_node_metrics",
        "validate_benchmark_local_essential_pipeline_fake_run_node_report",
        "validate_benchmark_local_essential_pipeline_fake_run_pipeline_report",
        "validate_benchmark_local_essential_pipeline_fake_runs_report",
    ] {
        assert!(contents.contains(validator), "thin CLI must invoke {validator}");
    }
    for forbidden in [
        "struct EssentialPipelineFakeRunOutputEntry {",
        "struct EssentialPipelineFakeRunInputBinding {",
        "struct EssentialPipelineFakeRunNodeMetrics {",
        "struct EssentialPipelineFakeRunNodeReport {",
        "struct EssentialPipelineFakeRunPipelineReport {",
        "struct EssentialPipelineFakeRunsReport {",
        "\"bijux.bench.local_essential_pipeline_fake_runs.v1\"",
        "\"bijux.bench.local_essential_pipeline_fake_run_pipeline.v1\"",
        "\"bijux.bench.local_essential_pipeline_fake_run_node_metrics.v1\"",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_pipeline_dag_validation_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_pipeline_dag.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_pipeline_dag_validation"));
    for forbidden in [
        "struct LocalPipelineReferenceContext {",
        "struct LocalPipelineExecutionContext {",
        "struct LocalPipelineProjectSource {",
        "struct LocalPipelineAncientDnaPolicy {",
        "struct LocalPipelineReferenceAssets {",
        "struct LocalPipelineVariantAssets {",
        "struct LocalPipelineSnpEffAssets {",
        "struct LocalPipelineDagValidationNodeReport {",
        "struct LocalPipelineDagValidationProfileReport {",
        "struct LocalPipelineDagValidationReport {",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
    assert!(!contents.contains("bijux.bench.local_pipeline_dag_validation.v1"));

    let aggregate_cli = repo_root.join("crates/bijux-dna/src/commands/fastq/meta/pipelines.rs");
    let aggregate_contents = fs::read_to_string(&aggregate_cli)
        .unwrap_or_else(|err| panic!("read {}: {err}", aggregate_cli.display()));
    assert!(aggregate_contents.contains("validate_benchmark_local_pipeline_dag_validation_set"));
    assert!(!aggregate_contents.contains("struct LocalPipelineDagValidationSetReport {"));
    assert!(!aggregate_contents.contains("bijux.bench.local_pipeline_dag_validation_set.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_slurm_dependency_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_slurm_dependency_check.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));
    assert!(contents.contains("validate_benchmark_local_slurm_dependency_check"));
    assert!(!contents.contains("struct BenchLocalSlurmDependencyCheckReport {"));
    assert!(!contents.contains("struct BenchLocalSlurmDependencyCheckJob {"));
    assert!(!contents.contains("bijux.bench.local_slurm_dependency_check.v1"));

    let shell_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_slurm_shell_syntax.rs");
    let shell_contents = fs::read_to_string(&shell_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", shell_source.display()));
    assert!(shell_contents.contains("validate_benchmark_local_slurm_shell_syntax"));
    assert!(!shell_contents.contains("struct BenchLocalSlurmShellSyntaxReport {"));
    assert!(!shell_contents.contains("struct BenchLocalSlurmShellSyntaxEntry {"));
    assert!(!shell_contents.contains("bijux.bench.local_slurm_shell_syntax.v1"));

    let dry_run_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_slurm_dry_run.rs");
    let dry_run_contents = fs::read_to_string(&dry_run_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", dry_run_source.display()));
    assert!(dry_run_contents.contains("validate_benchmark_local_slurm_dry_run"));
    assert!(!dry_run_contents.contains("struct BenchLocalSlurmDryRunReport {"));
    assert!(!dry_run_contents.contains("struct BenchLocalSlurmScriptEntry {"));
    assert!(!dry_run_contents.contains("bijux.bench.local_slurm_dry_run.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_stage_completion_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    for (file_name, validator, forbidden) in [
        (
            "local_stage_manifest_completion.rs",
            "validate_benchmark_local_stage_manifest_completion",
            "struct BenchLocalStageManifestCompletionReport",
        ),
        (
            "local_stage_output_completion.rs",
            "validate_benchmark_local_stage_output_completion",
            "struct BenchLocalStageOutputCompletionReport",
        ),
    ] {
        let cli_source = repo_root.join("crates/bijux-dna/src/commands/benchmark").join(file_name);
        let contents = fs::read_to_string(&cli_source)
            .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

        assert!(contents.contains(validator));
        assert!(!contents.contains(forbidden));
        assert!(!contents.contains("bijux.bench.local_stage_manifest_completion.v1"));
        assert!(!contents.contains("bijux.bench.local_stage_output_completion.v1"));
    }
}

#[test]
fn thin_cli_uses_model_owned_local_stage_command_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_commands.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));
    assert!(contents.contains("validate_benchmark_local_stage_commands"));
    for forbidden in [
        "struct BenchLocalStageArtifactEntry {",
        "struct BenchLocalStageCommandEntry {",
        "struct BenchLocalStageCommandManifest {",
        "struct BenchLocalStageCommandArgvEntry {",
        "bijux.bench.local_stage_commands.v3",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_slurm_submit_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let submit_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_slurm_submit_manifest.rs");
    let submit_contents = fs::read_to_string(&submit_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", submit_source.display()));
    assert!(submit_contents.contains("validate_benchmark_local_slurm_submit_manifest"));
    for forbidden in [
        "struct BenchLocalSlurmSubmitManifest {",
        "struct BenchLocalSlurmSubmitJob {",
        "struct BenchLocalSlurmSubmitResources {",
        "struct BenchLocalSlurmSubmitLogs {",
        "bijux.bench.local_slurm_submit_manifest.v1",
    ] {
        assert!(!submit_contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }

    let dependency_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_slurm_dependency_check.rs");
    let dependency_contents = fs::read_to_string(&dependency_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", dependency_source.display()));
    assert!(!dependency_contents.contains("struct BenchLocalSlurmSubmitManifestRecord {"));
    assert!(!dependency_contents.contains("struct BenchLocalSlurmSubmitJobRecord {"));
}

#[test]
fn thin_cli_uses_model_owned_all_domain_slurm_submit_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_all_domain_slurm_submit_manifest.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));
    assert!(contents.contains("validate_benchmark_local_all_domain_slurm_submit_manifest"));
    for forbidden in [
        "struct BenchLocalAllDomainSlurmSubmitManifest {",
        "struct BenchLocalAllDomainSlurmSubmitJob {",
        "struct BenchLocalAllDomainSlurmSubmitResources {",
        "bijux.bench.local_all_domain_slurm_submit_manifest.v1",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_candidate_run_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_hpc_candidate_run_manifest.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));
    assert!(contents.contains("validate_benchmark_local_hpc_candidate_run_manifest"));
    for forbidden in [
        "struct LocalHpcCandidateResourceCeiling {",
        "struct LocalHpcCandidateBehavior {",
        "struct LocalHpcCandidateStopCondition {",
        "struct LocalHpcCandidateResourceHint {",
        "struct LocalHpcCandidateJob {",
        "struct LocalHpcCandidateRunManifest {",
        "bijux.bench.local_hpc_candidate_run_manifest.v1",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_execution_resolver_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_execution_resolver.rs");
    let contents = fs::read_to_string(&source)
        .unwrap_or_else(|err| panic!("read {}: {err}", source.display()));
    assert!(contents.contains("validate_benchmark_local_hpc_execution_resolver"));
    for forbidden in [
        "struct LocalHpcExecutionResolverRow {",
        "struct LocalHpcExecutionResolverReport {",
        "bijux.bench.local_hpc_execution_resolver.v1",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_local_stage_inventory_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_inventory.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_stage_inventory"));
    assert!(contents.contains("validate_benchmark_local_all_domain_stage_inventory"));
    assert!(!contents.contains("struct BenchLocalStageInventory"));
    assert!(!contents.contains("struct BenchLocalAllDomainStageInventory"));
    assert!(!contents.contains("bijux.bench.local_stage_inventory.v1"));
    assert!(!contents.contains("bijux.bench.local_all_domain_stage_inventory.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_job_graph_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_job_graph.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_job_graph"));
    assert!(!contents.contains("struct LocalHpcJobGraphNode"));
    assert!(!contents.contains("struct LocalHpcJobGraph"));
    assert!(!contents.contains("bijux.bench.local_hpc_job_graph.v1"));
}

#[test]
fn thin_cli_uses_model_owned_fastq_corpus_run_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/corpus_fastq");
    let command = fs::read_to_string(cli_root.join("mod.rs")).expect("read corpus command");
    let local_models = fs::read_to_string(cli_root.join("models.rs")).expect("read CLI models");

    assert!(command.contains("validate_benchmark_local_corpus_run"));
    assert!(command.contains("BenchmarkCorpusRunManifest"));
    assert!(!local_models.contains("struct CorpusRunManifest"));
    assert!(!local_models.contains("struct SampleRunRecord"));
    assert!(!local_models.contains("bijux.fastq.corpus_benchmark_run.v1"));
}

#[test]
fn thin_cli_uses_model_owned_published_results_status_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command = fs::read_to_string(cli_root.join("results_status.rs"))
        .expect("read published-results status command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("validate_benchmark_published_results_status"));
    assert!(command.contains("PublishedResultsStatusReport"));
    assert!(!local_models.contains("struct PublishedResultsStatusReport"));
    assert!(!local_models.contains("struct PublishedResultsStageReport"));
    assert!(!local_models.contains("struct StageResultIssue"));
}

#[test]
fn benchmark_owner_uses_model_owned_dossier_index_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let owner =
        fs::read_to_string(repo_root.join("crates/bijux-dna-bench/src/workflow/dossier_index.rs"))
            .expect("read dossier-index owner");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(owner.contains("validate_benchmark_dossier_index"));
    assert!(owner.contains("BenchmarkDossierIndex"));
    assert!(owner.contains("validate_benchmark_corpus_dossier_summary"));
    assert!(owner.contains("BenchmarkCorpusDossierSummary"));
    assert!(!local_models.contains("struct DossierIndex"));
    assert!(!local_models.contains("struct DossierStageEntry"));
}

#[test]
fn thin_cli_uses_model_owned_publication_status_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command = fs::read_to_string(cli_root.join("docs_status.rs"))
        .expect("read publication-status command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("BenchmarkPublicationStatusReport"));
    assert!(command.contains("validate_benchmark_publication_status"));
    assert!(!local_models.contains("struct BenchmarkPublicationStatusReport"));
    assert!(!local_models.contains("struct PublicationStageReport"));
    assert!(!local_models.contains("struct StageAuditIssue"));
}

#[test]
fn thin_cli_uses_model_owned_remediation_queue_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command =
        fs::read_to_string(cli_root.join("remediation.rs")).expect("read remediation command");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(command.contains("RemediationQueue"));
    assert!(command.contains("validate_benchmark_remediation_queue"));
    assert!(!local_models.contains("struct RemediationQueue"));
    assert!(!local_models.contains("struct RemediationStageEntry"));
    assert!(!local_models.contains("struct RemediationIssueGroup"));
    assert!(!local_models.contains("struct RemediationIssue"));
}

#[test]
fn thin_cli_uses_model_owned_publication_findings_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let command =
        fs::read_to_string(cli_root.join("docs_status.rs")).expect("read findings command");

    assert!(command.contains("BenchmarkPublicationFindings"));
    assert!(command.contains("validate_benchmark_publication_findings"));
    assert!(!command.contains("payload.get(\"findings\")"));
}

#[test]
fn thin_cli_uses_model_owned_fastq_corpus_source_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let corpus_metadata = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/corpus_metadata.rs"),
    )
    .expect("read corpus metadata command");
    let publication_models = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/publication/models.rs"),
    )
    .expect("read publication CLI models");

    assert!(corpus_metadata.contains("load_fastq_corpus_source_spec"));
    assert!(corpus_metadata.contains("FastqCorpusSourceSpec as CorpusSpec"));
    assert!(!corpus_metadata.contains("struct CorpusSpec"));
    assert!(!corpus_metadata.contains("struct CorpusSpecSample"));
    assert!(!publication_models.contains("struct PublicationCorpusSpec"));
}

#[test]
fn thin_cli_uses_model_owned_corpus_dossier_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/publication");
    let dossier =
        fs::read_to_string(cli_root.join("corpus_dossier.rs")).expect("read corpus dossier");
    let local_models =
        fs::read_to_string(cli_root.join("models.rs")).expect("read publication CLI models");

    assert!(dossier.contains("validate_benchmark_corpus_dossier_summary"));
    assert!(dossier.contains("BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1"));
    for forbidden in [
        "struct CorpusSummary",
        "struct CorpusToolSummary",
        "struct CorpusHeadline",
        "struct CorpusSampleResultRow",
    ] {
        assert!(!local_models.contains(forbidden));
    }
    assert!(!dossier.contains("bijux.corpus_benchmark.summary.v1"));
}

#[test]
fn thin_cli_uses_model_owned_comparable_metric_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "fastq_comparable_metrics.rs",
            "validate_fastq_comparable_metrics_report",
            ["struct FastqComparableMetricsRow", "struct FastqComparableMetricsReport"],
            "bijux.bench.readiness.fastq_comparable_metrics.v1",
        ),
        (
            "bam_comparable_metrics.rs",
            "validate_bam_comparable_metrics_report",
            ["struct BamComparableMetricsRow", "struct BamComparableMetricsReport"],
            "bijux.bench.readiness.bam_comparable_metrics.v1",
        ),
        (
            "vcf_comparable_metrics.rs",
            "validate_vcf_comparable_metrics_report",
            ["struct VcfComparableMetricsRow", "struct VcfComparableMetricsReport"],
            "bijux.bench.readiness.vcf_comparable_metrics.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read readiness command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
        if file_name == "all_domain_active_stage_tool_matrix.rs" {
            for licensed_tool in ["\"angsd\"", "\"fastuniq\"", "\"impute5\"", "\"ngsbriggs\""] {
                assert!(!source.contains(licensed_tool));
            }
        }
    }
}

#[test]
fn thin_cli_uses_model_owned_readiness_view_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "stage_centric_report.rs",
            "validate_stage_centric_report",
            [
                "struct StageCentricToolRow",
                "struct StageCentricStageReport",
                "struct StageCentricReport",
            ],
            "bijux.bench.readiness.stage_centric_report.v1",
        ),
        (
            "tool_centric_report.rs",
            "validate_tool_centric_report",
            [
                "struct ToolCentricStageRow",
                "struct ToolCentricToolReport",
                "struct ToolCentricReport",
            ],
            "bijux.bench.readiness.tool_centric_report.v1",
        ),
        (
            "corpus_centric_report.rs",
            "validate_corpus_centric_report",
            [
                "struct CorpusCentricStageRow",
                "struct CorpusCentricCorpusReport",
                "struct CorpusCentricReport",
            ],
            "bijux.bench.readiness.corpus_centric_report.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read readiness command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
    }
}

#[test]
fn thin_cli_uses_model_owned_stage_tool_benchmark_ready_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let source =
        fs::read_to_string(repo_root.join(
            "crates/bijux-dna/src/commands/benchmark/readiness/stage_tool_benchmark_ready.rs",
        ))
        .expect("read stage/tool benchmark-ready command");

    assert!(source.contains("validate_stage_tool_benchmark_ready_report"));
    assert!(source.contains("STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1"));
    for forbidden in [
        "struct StageToolBenchmarkReadySurfaceSummary",
        "struct StageToolBenchmarkReadyFailingPair",
        "struct StageToolBenchmarkReadyExcludedPair",
        "struct StageToolBenchmarkReadyReport",
        "bijux.bench.readiness.fastq_bam_stage_tool_benchmark_ready.v1",
    ] {
        assert!(!source.contains(forbidden));
    }
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_stage_catalog_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let source = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_stage_catalog.rs"),
    )
    .expect("read local VCF stage-catalog command");

    assert!(source.contains("render_vcf_stage_catalog"));
    assert!(source.contains("DEFAULT_VCF_STAGE_CATALOG_PATH"));
    for forbidden in [
        "struct LocalVcfStageCatalogConfig",
        "struct VcfStageCatalogRow",
        "struct LocalVcfStageCatalogReport",
        "bijux.bench.vcf.local_stage_catalog.v1",
        "bijux.bench.local_vcf_stage_catalog.v1",
        "vcf_stage_catalog()",
        "stage_io_contract",
    ] {
        assert!(!source.contains(forbidden));
    }
}

#[test]
fn thin_cli_uses_model_owned_local_vcf_stage_matrix_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let source = fs::read_to_string(
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_vcf_stage_matrix.rs"),
    )
    .expect("read local VCF stage-matrix command");

    for validator in ["render_vcf_stage_matrix", "validate_vcf_stage_matrix"] {
        assert!(source.contains(validator));
    }
    for forbidden in [
        "struct LocalVcfStageMatrixConfig",
        "struct VcfStageMatrixRow",
        "struct LocalVcfStageMatrixReport",
        "struct ValidateMatrixReport",
        "bijux.bench.vcf.local_stage_matrix.v1",
        "bijux.bench.local_vcf_stage_matrix.v1",
        "bijux.bench.validate_matrix.v1",
        "vcf_domain_stage_adapter_id",
        "load_required_tool_ids",
    ] {
        assert!(!source.contains(forbidden));
    }
}

#[test]
fn thin_cli_uses_model_owned_scope_view_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_root = repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness");
    for (file_name, validator, forbidden_types, forbidden_schema) in [
        (
            "all_domain_active_stage_tool_matrix.rs",
            "validate_all_domain_active_stage_tool_matrix",
            [
                "struct AllDomainActiveStageToolMatrixRow",
                "struct AllDomainActiveStageToolMatrixReport",
            ],
            "bijux.bench.readiness.all_domain_active_stage_tool_matrix.v1",
        ),
        (
            "all_domain_retained_tools.rs",
            "validate_all_domain_retained_tools",
            ["struct AllDomainRetainedToolRow", "struct AllDomainRetainedToolsReport"],
            "bijux.bench.readiness.retained_tool_inventory.v1",
        ),
        (
            "stage_registry_extra_pairs.rs",
            "validate_stage_registry_extra_pairs",
            ["struct StageRegistryExtraPairRow", "struct StageRegistryExtraPairsReport"],
            "bijux.bench.readiness.stage_registry_extra_pairs.v1",
        ),
    ] {
        let source = fs::read_to_string(cli_root.join(file_name)).expect("read scope-view command");
        assert!(source.contains(validator));
        for forbidden_type in forbidden_types {
            assert!(!source.contains(forbidden_type));
        }
        assert!(!source.contains(forbidden_schema));
    }
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_resume_simulation_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_resume_simulation.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_resume_simulation"));
    assert!(!contents.contains("struct LocalHpcResumeSimulationBehavior {"));
    assert!(!contents.contains("struct LocalHpcResumeSimulationJob {"));
    assert!(!contents.contains("struct LocalHpcResumeSimulationReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_resume_simulation.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_dependency_simulation_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_hpc_dependency_simulation.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_dependency_simulation"));
    assert!(!contents.contains("struct LocalHpcDependencySimulationJob {"));
    assert!(!contents.contains("struct LocalHpcDependencySimulationCase {"));
    assert!(!contents.contains("struct LocalHpcDependencySimulationReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_dependency_simulation.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_stage_benchmark_array_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/local_hpc_stage_benchmark_array.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_stage_benchmark_array"));
    assert!(!contents.contains("struct LocalHpcStageBenchmarkArrayRow {"));
    assert!(!contents.contains("struct LocalHpcStageBenchmarkArrayReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_stage_benchmark_array.v1\""));

    let support_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_array_support.rs");
    let support = fs::read_to_string(&support_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", support_source.display()));
    assert!(!support.contains("struct LocalHpcArrayResources {"));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_pipeline_node_array_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_pipeline_node_array.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_pipeline_node_array"));
    assert!(!contents.contains("struct LocalHpcPipelineNodeArrayExpectedOutput {"));
    assert!(!contents.contains("struct LocalHpcPipelineNodeArrayDependency {"));
    assert!(!contents.contains("struct LocalHpcPipelineNodeArrayRow {"));
    assert!(!contents.contains("struct LocalHpcPipelineNodeArrayReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_pipeline_node_array.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_dry_run_readiness_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_dry_run_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_dry_run_readiness"));
    assert!(!contents.contains("struct LocalHpcDryRunReadyGoalCheck {"));
    assert!(!contents.contains("struct LocalHpcDryRunReadySummary {"));
    assert!(!contents.contains("struct LocalHpcDryRunReadyBehavior {"));
    assert!(!contents.contains("struct LocalHpcDryRunReadyReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_dry_run_ready.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_local_hpc_submission_readiness_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_hpc_submission_ready.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_hpc_submission_readiness"));
    assert!(!contents.contains("struct BenchLocalHpcSubmissionReadyGoalCheck {"));
    assert!(!contents.contains("struct BenchLocalHpcSubmissionReadyReport {"));
    assert!(!contents.contains("\"bijux.bench.local_hpc_submission_ready.v1\""));
}

#[test]
fn thin_cli_uses_model_owned_local_stage_fake_run_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_fake_runs.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    for validator in [
        "validate_benchmark_local_stage_fake_run_manifest",
        "validate_benchmark_local_stage_fake_run_result",
        "validate_benchmark_local_stage_fake_failure_manifest",
        "validate_benchmark_local_stage_fake_failure_record",
    ] {
        assert!(contents.contains(validator), "thin CLI must invoke {validator}");
    }
    for forbidden in [
        "struct BenchLocalStageFakeRunOutputEntry {",
        "struct BenchLocalStageFakeRunResult {",
        "struct BenchLocalStageFakeRunManifest {",
        "struct BenchLocalStageFakeFailureOutputEntry {",
        "struct BenchLocalStageFakeFailureRecord {",
        "struct BenchLocalStageFakeFailureManifest {",
        "\"bijux.bench.local_stage_fake_runs.v1\"",
        "\"bijux.bench.local_stage_fake_failures.v1\"",
        "\"bijux.bench.local_stage_fake_failure_record.v1\"",
    ] {
        assert!(!contents.contains(forbidden), "thin CLI reintroduced {forbidden}");
    }
}

#[test]
fn thin_cli_uses_model_owned_scientific_acceptance_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root.join(
        "crates/bijux-dna/src/commands/benchmark/readiness/scientific_acceptance_thresholds.rs",
    );
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_scientific_acceptance_thresholds_config"));
    assert!(contents.contains("validate_scientific_acceptance_thresholds_report"));
    assert!(!contents.contains("struct ScientificAcceptanceThresholdRow {"));
    assert!(!contents.contains("struct ScientificAcceptanceThresholdsConfig {"));
    assert!(!contents.contains("struct ScientificAcceptanceThresholdsReport {"));
    assert!(!contents.contains("enum ScientificAcceptanceDirection {"));
    assert!(!contents.contains("bijux.bench.local_scientific_acceptance_thresholds.v1"));
    assert!(!contents.contains("bijux.bench.readiness.scientific_acceptance_thresholds.v1"));
}

#[test]
fn thin_cli_uses_model_owned_expected_benchmark_result_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root.join(
        "crates/bijux-dna/src/commands/benchmark/readiness/all_domain_expected_benchmark_results.rs",
    );
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_all_domain_expected_benchmark_results"));
    assert!(!contents.contains("struct AllDomainExpectedBenchmarkResultRow {"));
    assert!(!contents.contains("struct AllDomainExpectedBenchmarkResultsReport {"));
    assert!(!contents.contains("bijux.bench.readiness.all_domain_expected_benchmark_results.v1"));
}

#[test]
fn thin_cli_uses_model_owned_stage_tool_resource_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness/stage_tool_resources.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_stage_tool_resources_config"));
    assert!(contents.contains("validate_stage_tool_resources_report"));
    assert!(!contents.contains("struct StageToolResourceRow {"));
    assert!(!contents.contains("struct StageToolResourcesConfig {"));
    assert!(!contents.contains("struct StageToolResourcesReport {"));
    assert!(!contents.contains("bijux.bench.local_stage_tool_resources.v1"));
}

#[test]
fn thin_cli_uses_model_owned_stage_tool_asset_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/readiness/stage_tool_assets.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_stage_tool_assets_config"));
    assert!(contents.contains("validate_stage_tool_assets_report"));
    assert!(!contents.contains("struct StageToolAssetRow {"));
    assert!(!contents.contains("struct StageToolAssetsConfig {"));
    assert!(!contents.contains("struct StageToolAssetsReport {"));
    assert!(!contents.contains("bijux.bench.local_stage_tool_assets.v1"));
}

#[test]
fn thin_cli_uses_model_owned_stage_tool_container_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root
        .join("crates/bijux-dna/src/commands/benchmark/readiness/stage_tool_containers.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_stage_tool_containers_config"));
    assert!(contents.contains("validate_stage_tool_containers_report"));
    assert!(!contents.contains("struct StageToolContainerRow {"));
    assert!(!contents.contains("struct StageToolContainersConfig {"));
    assert!(!contents.contains("struct StageToolContainersReport {"));
    assert!(!contents.contains("bijux.bench.local_stage_tool_containers.v1"));
}

#[test]
fn thin_cli_uses_model_owned_all_domain_output_declaration_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source = repo_root.join(
        "crates/bijux-dna/src/commands/benchmark/readiness/all_domain_output_declarations.rs",
    );
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_all_domain_output_declaration_rows"));
    assert!(contents.contains("validate_all_domain_output_declarations_report"));
    assert!(!contents.contains("enum AllDomainOutputDeclarationStatus {"));
    assert!(!contents.contains("struct AllDomainOutputDeclarationRow {"));
    assert!(!contents.contains("struct AllDomainOutputDeclarationsReport {"));
    assert!(!contents.contains("bijux.bench.readiness.all_domain_output_declarations.v1"));
}

fn collect_rs_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
