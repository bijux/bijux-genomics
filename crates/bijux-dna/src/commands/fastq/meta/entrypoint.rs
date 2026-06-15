use super::analyze::handle_analyze_command;
use super::debug::handle_debug_command;
use super::environment::handle_environment_command;
use super::pipelines::handle_pipelines_command;
use crate::cli::BenchConfigCommand;
use crate::commands::fastq::api_bridge::{
    bench_bam_pipeline_args_to_api, bench_bam_stage_args_to_api,
};
#[allow(unused_imports)]
use crate::commands::support::prelude::{
    anyhow, atomic_write_bytes, bench_args_cluster_otus, bench_args_correct,
    bench_args_deplete_host, bench_args_deplete_reference_contaminants, bench_args_deplete_rrna,
    bench_args_detect_adapters, bench_args_filter, bench_args_filter_low_complexity,
    bench_args_index_reference, bench_args_infer_asvs, bench_args_merge,
    bench_args_normalize_abundance, bench_args_normalize_primers, bench_args_preprocess,
    bench_args_profile_overrepresented, bench_args_profile_read_lengths, bench_args_qc_post,
    bench_args_remove_chimeras, bench_args_remove_duplicates, bench_args_screen, bench_args_stats,
    bench_args_trim, bench_args_trim_polyg, bench_args_trim_terminal_damage, bench_args_umi,
    bench_args_validate, bench_fastq_cluster_otus, bench_fastq_correct, bench_fastq_deplete_host,
    bench_fastq_deplete_reference_contaminants, bench_fastq_deplete_rrna,
    bench_fastq_detect_adapters, bench_fastq_filter, bench_fastq_filter_low_complexity,
    bench_fastq_index_reference, bench_fastq_infer_asvs, bench_fastq_merge,
    bench_fastq_normalize_abundance, bench_fastq_normalize_primers, bench_fastq_preprocess,
    bench_fastq_profile_overrepresented, bench_fastq_profile_read_lengths, bench_fastq_qc_post,
    bench_fastq_remove_chimeras, bench_fastq_remove_duplicates, bench_fastq_screen,
    bench_fastq_stats_neutral, bench_fastq_trim, bench_fastq_trim_polyg_tails,
    bench_fastq_trim_terminal_damage, bench_fastq_umi, bench_fastq_validate_reads, cli,
    load_image_catalog, load_manifests, load_platform, print_bench_schema, qc_class_label, render,
    run_image_qa, set_tool_tier_policy, workspace_audit, write_chimeras_report,
    write_cluster_otus_report, write_correct_report, write_deplete_host_report,
    write_deplete_reference_contaminants_report, write_deplete_rrna_report,
    write_detect_adapters_report, write_duplicates_report, write_filter_low_complexity_report,
    write_filter_report, write_index_reference_report, write_infer_asvs_report, write_merge_report,
    write_normalize_abundance_report, write_normalize_primers_report, write_overrepresented_report,
    write_qc_post_report, write_read_lengths_report, write_run_report_from_facts,
    write_run_summary_from_facts, write_screen_report, write_stage_summary_csv, write_stats_report,
    write_trim_polyg_report, write_trim_report, write_trim_terminal_damage_report,
    write_umi_report, write_validate_report, BenchBamCommand, BenchCommand, BenchFastqCommand, Cli,
    DnaCommand, Objective, Path, PoliciesCommand, Result,
};

#[allow(clippy::too_many_lines)]
pub(crate) fn handle_meta_commands(
    cli: &Cli,
    dna_command: &DnaCommand,
    _domain_dir: &Path,
    registry_path: &Path,
) -> Result<bool> {
    if let Some(done) = handle_debug_command(cli, dna_command, registry_path)? {
        return Ok(done);
    }

    match dna_command {
        DnaCommand::Pipelines(args) => handle_pipelines_command(args, registry_path),
        DnaCommand::Analyze(args) | DnaCommand::Explain(args) => handle_analyze_command(args),
        DnaCommand::Environment(args) => handle_environment_command(cli, args),
        DnaCommand::Bench(args) => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match &args.command {
                BenchCommand::Config { command } => match command {
                    BenchConfigCommand::Validate(args) => {
                        crate::commands::benchmark_config::validate_benchmark_config(
                            &std::env::current_dir()?,
                            args,
                        )?;
                    }
                },
                BenchCommand::Run(args) => {
                    let run_dir = crate::commands::bench_suite::run_suite(
                        &std::env::current_dir()?,
                        &args.suite,
                        args.hpc,
                    )?;
                    println!("suite_run_dir={}", run_dir.display());
                }
                BenchCommand::Status => {
                    let cwd = std::env::current_dir()?;
                    let status = crate::commands::bench_suite::bench_status(&cwd);
                    crate::commands::cli::render::json::print_pretty(&status)?;
                }
                BenchCommand::WorkspaceValue(args) => {
                    crate::commands::benchmark_corpus_fastq::print_benchmark_workspace_value(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::ConfigJson(args) => {
                    crate::commands::benchmark_workspace::print_benchmark_config_json(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::RepoChecks(args) => {
                    crate::commands::benchmark_repo_checks::run_benchmark_repo_checks_command(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::ValidateMatrix(args) => {
                    let cwd = std::env::current_dir()?;
                    match args.domain {
                        cli::BenchMatrixDomainArg::Vcf => {
                            crate::commands::benchmark::local_vcf_stage_matrix::run_validate_vcf_stage_matrix(
                                &cwd,
                                args,
                            )?;
                        }
                    }
                }
                BenchCommand::ValidateSchemas(args) => {
                    let cwd = std::env::current_dir()?;
                    crate::commands::benchmark::schema_validation::run_validate_schemas(
                        &cwd, args,
                    )?;
                }
                BenchCommand::ActiveScope { command } => match command {
                    cli::BenchActiveScopeCommand::Validate(args) => {
                        crate::commands::benchmark::active_scope::run_active_scope_validate_command(
                            &std::env::current_dir()?,
                            args,
                        )?;
                    }
                },
                BenchCommand::Paths { command } => match command {
                    cli::BenchPathsCommand::Validate(args) => {
                        crate::commands::benchmark_paths::run_benchmark_paths_validate_command(
                            &std::env::current_dir()?,
                            args,
                        )?;
                    }
                    cli::BenchPathsCommand::ProveDisposableRootCleanup(args) => {
                        crate::commands::benchmark_paths::run_disposable_root_cleanup_proof_command(
                            &std::env::current_dir()?,
                            args,
                        )?;
                    }
                },
                BenchCommand::WriteScreenTaxonomyDatabaseLineage(args) => {
                    crate::commands::benchmark_taxonomy_database::run_write_screen_taxonomy_database_lineage(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::NormalizeWorkspaceLayout(args) => {
                    crate::commands::benchmark_workspace::run_normalize_workspace_layout(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::PublicationTargets(args) => {
                    crate::commands::benchmark_publication::print_benchmark_publication_targets(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::CorpusFastqReport(args) => {
                    crate::commands::benchmark_publication::run_corpus_fastq_report(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::CorpusFastqPublicationStatus(args) => {
                    crate::commands::benchmark_publication::run_corpus_fastq_publication_status(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::CorpusFastqPublishedDossiers(args) => {
                    crate::commands::benchmark_publication::run_corpus_fastq_published_dossiers(
                        &std::env::current_dir()?,
                        args,
                    )?;
                }
                BenchCommand::Readiness { command } => match command {
                    cli::BenchReadinessCommand::RenderAdapterMissingInputTests(args) => {
                        crate::commands::benchmark::readiness::adapter_missing_input_tests::run_render_adapter_missing_input_tests(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderCommands(args) => {
                        crate::commands::benchmark::readiness::rendered_commands::run_render_commands(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderCommandArgv(args) => {
                        crate::commands::benchmark::readiness::rendered_command_argv::run_render_command_argv(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageToolContainers(args) => {
                        crate::commands::benchmark::readiness::stage_tool_containers::run_render_stage_tool_containers(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageToolAssets(args) => {
                        crate::commands::benchmark::readiness::stage_tool_assets::run_render_stage_tool_assets(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageToolResources(args) => {
                        crate::commands::benchmark::readiness::stage_tool_resources::run_render_stage_tool_resources(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamStageDecisionTable(args) => {
                        crate::commands::benchmark::readiness::bam_stage_decision_table::run_render_bam_stage_decision_table(
                        args,
                    )?;
                    }
                    cli::BenchReadinessCommand::RenderBamAdapterOutputContract(args) => {
                        crate::commands::benchmark::readiness::bam_adapter_output_contract::run_render_bam_adapter_output_contract(
                        args,
                    )?;
                    }
                    cli::BenchReadinessCommand::RenderBamCommandAdapterCoverage(args) => {
                        crate::commands::benchmark::readiness::bam_command_adapter_coverage::run_render_bam_command_adapter_coverage(
                        args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamCorpusAssignment(args) => {
                        crate::commands::benchmark::readiness::bam_corpus_assignment::run_render_bam_corpus_assignment(
                        args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderCorpusIncompatibility(args) => {
                        crate::commands::benchmark::readiness::corpus_incompatibility::run_render_corpus_incompatibility(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderCorpusCentricReport(args) => {
                        crate::commands::benchmark::readiness::corpus_centric_report::run_render_corpus_centric_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBenchmarkReadinessDashboard(args) => {
                        crate::commands::benchmark::readiness::benchmark_readiness_dashboard::run_render_benchmark_readiness_dashboard(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageToolBenchmarkReady(args) => {
                        crate::commands::benchmark::readiness::stage_tool_benchmark_ready::run_render_stage_tool_benchmark_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamComparableMetrics(args) => {
                        crate::commands::benchmark::readiness::bam_comparable_metrics::run_render_bam_comparable_metrics(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamNormalizedMetricsSchema(args) => {
                        crate::commands::benchmark::readiness::bam_normalized_metrics_schema::run_render_bam_normalized_metrics_schema(
                        args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamParserCoverage(args) => {
                        crate::commands::benchmark::readiness::bam_parser_coverage::run_render_bam_parser_coverage(
                        args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamReportMap(args) => {
                        crate::commands::benchmark::readiness::bam_report_map::run_render_bam_report_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderExpectedBenchmarkResults(args) => {
                        crate::commands::benchmark::readiness::expected_benchmark_results::run_render_expected_benchmark_results(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderMissingResultReport(args) => {
                        crate::commands::benchmark::readiness::missing_result_report::run_render_missing_result_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderPairReadiness(args) => {
                        crate::commands::benchmark::readiness::pair_readiness::run_render_pair_readiness(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageCentricReport(args) => {
                        crate::commands::benchmark::readiness::stage_centric_report::run_render_stage_centric_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderToolCentricReport(args) => {
                        crate::commands::benchmark::readiness::tool_centric_report::run_render_tool_centric_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderParserCompletenessGate(args) => {
                        crate::commands::benchmark::readiness::parser_completeness_gate::run_render_parser_completeness_gate(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderCorpusAssetCoverageGate(args) => {
                        crate::commands::benchmark::readiness::corpus_asset_coverage_gate::run_render_corpus_asset_coverage_gate(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelineCorpusAssets(args) => {
                        crate::commands::benchmark::readiness::essential_pipeline_corpus_assets::run_render_essential_pipeline_corpus_assets(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelineFailureIsolation(args) => {
                        crate::commands::benchmark::readiness::essential_pipeline_failure_isolation::run_render_essential_pipeline_failure_isolation(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelinesReady(args) => {
                        crate::commands::benchmark::readiness::essential_pipelines_ready::run_render_essential_pipelines_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelineReportMap(args) => {
                        crate::commands::benchmark::readiness::essential_pipeline_report_map::run_render_essential_pipeline_report_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelinePartialResume(args) => {
                        crate::commands::benchmark::readiness::essential_pipeline_partial_resume::run_render_essential_pipeline_partial_resume(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderEssentialPipelineCommands(args) => {
                        crate::commands::benchmark::readiness::essential_pipeline_rendered_commands::run_render_essential_pipeline_commands(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderParserFailureTests(args) => {
                        crate::commands::benchmark::readiness::parser_failure_tests::run_render_parser_failure_tests(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqAdapterOutputContract(args) => {
                        crate::commands::benchmark::readiness::fastq_adapter_output_contract::run_render_fastq_adapter_output_contract(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqCommandAdapterCoverage(args) => {
                        crate::commands::benchmark::readiness::fastq_command_adapter_coverage::run_render_fastq_command_adapter_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqComparableMetrics(args) => {
                        crate::commands::benchmark::readiness::fastq_comparable_metrics::run_render_fastq_comparable_metrics(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqCorpusAssignment(args) => {
                        crate::commands::benchmark::readiness::fastq_corpus_assignment::run_render_fastq_corpus_assignment(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqNormalizedMetricsSchema(args) => {
                        crate::commands::benchmark::readiness::fastq_normalized_metrics_schema::run_render_fastq_normalized_metrics_schema(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqParserCoverage(args) => {
                        crate::commands::benchmark::readiness::fastq_parser_coverage::run_render_fastq_parser_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqParserFixtureCoverage(args) => {
                        crate::commands::benchmark::readiness::fastq_parser_fixture_coverage::run_render_fastq_parser_fixture_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqCommands(args) => {
                        crate::commands::benchmark::readiness::fastq_rendered_commands::run_render_fastq_commands(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqReportMap(args) => {
                        crate::commands::benchmark::readiness::fastq_report_map::run_render_fastq_report_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqActiveStageToolMatrix(args) => {
                        crate::commands::benchmark::readiness::fastq_active_stage_tool_matrix::run_render_fastq_active_stage_tool_matrix(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqLocalContainerSmoke(args) => {
                        crate::commands::benchmark::readiness::fastq_local_container_smoke::run_render_fastq_local_container_smoke(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqDuplicateStagesReady(args) => {
                        crate::commands::benchmark::readiness::fastq_duplicate_stages_ready::run_render_fastq_duplicate_stages_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqFilterStagesReady(args) => {
                        crate::commands::benchmark::readiness::fastq_filter_stages_ready::run_render_fastq_filter_stages_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqTrimStagesReady(args) => {
                        crate::commands::benchmark::readiness::fastq_trim_stages_ready::run_render_fastq_trim_stages_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqValidateReadsReady(args) => {
                        crate::commands::benchmark::readiness::fastq_validate_reads_ready::run_render_fastq_validate_reads_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFastqToolServingMap(args) => {
                        crate::commands::benchmark::readiness::tool_serving_map::run_render_fastq_tool_serving_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamToolServingMap(args) => {
                        crate::commands::benchmark::readiness::tool_serving_map::run_render_bam_tool_serving_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfToolServingMap(args) => {
                        crate::commands::benchmark::readiness::vcf_tool_serving_map::run_render_vcf_tool_serving_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainExpectedBenchmarkResults(args) => {
                        crate::commands::benchmark::readiness::all_domain_expected_benchmark_results::run_render_all_domain_expected_benchmark_results(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainExpectedResultCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_expected_result_coverage::run_render_all_domain_expected_result_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainHarnessReady(args) => {
                        crate::commands::benchmark::readiness::all_domain_harness_ready::run_render_all_domain_harness_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainLocalJobCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_local_job_coverage::run_render_all_domain_local_job_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainNoPlaceholderCommandCheck(args) => {
                        crate::commands::benchmark::readiness::all_domain_no_placeholder_command_check::run_render_all_domain_no_placeholder_command_check(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainFailureClassification(args) => {
                        crate::commands::benchmark::readiness::all_domain_failure_classification::run_render_all_domain_failure_classification(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainCompletionCheck(args) => {
                        crate::commands::benchmark::readiness::all_domain_completion_check::run_render_all_domain_completion_check(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainMissingResultTest(args) => {
                        crate::commands::benchmark::readiness::all_domain_missing_result_test::run_render_all_domain_missing_result_test(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainParserCollector(args) => {
                        crate::commands::benchmark::readiness::all_domain_parser_collector::run_render_all_domain_parser_collector(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFullBenchmarkResultCollector(args) => {
                        crate::commands::benchmark::readiness::full_benchmark_result_collector::run_render_full_benchmark_result_collector(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFullBenchmarkDashboard(args) => {
                        crate::commands::benchmark::readiness::full_benchmark_dashboard::run_render_full_benchmark_dashboard(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderFullBenchmarkReport(args) => {
                        crate::commands::benchmark::readiness::full_benchmark_report::run_render_full_benchmark_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderOperationalBenchmarkReady(args) => {
                        crate::commands::benchmark::readiness::operational_benchmark_ready::run_render_operational_benchmark_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainOutputDeclarations(args) => {
                        crate::commands::benchmark::readiness::all_domain_output_declarations::run_render_all_domain_output_declarations(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainCommands(args) => {
                        crate::commands::benchmark::readiness::all_domain_rendered_commands::run_render_all_domain_commands(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainActiveStageCatalog(args) => {
                        crate::commands::benchmark::readiness::all_domain_active_stage_catalog::run_render_all_domain_active_stage_catalog(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainActiveScopeBlockers(args) => {
                        crate::commands::benchmark::readiness::all_domain_active_scope_blockers::run_render_all_domain_active_scope_blockers(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainActiveScopeComplete(args) => {
                        crate::commands::benchmark::readiness::all_domain_active_scope_complete::run_render_all_domain_active_scope_complete(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainAdapterCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_adapter_coverage::run_render_all_domain_adapter_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainOutputContractCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_output_contract_coverage::run_render_all_domain_output_contract_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainParserFixtureCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_parser_fixture_coverage::run_render_all_domain_parser_fixture_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainReportMapCoverage(args) => {
                        crate::commands::benchmark::readiness::all_domain_report_map_coverage::run_render_all_domain_report_map_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainActiveStageToolMatrix(args) => {
                        crate::commands::benchmark::readiness::all_domain_active_stage_tool_matrix::run_render_all_domain_active_stage_tool_matrix(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainNoDeclaredOnlyRows(args) => {
                        crate::commands::benchmark::readiness::all_domain_no_declared_only_rows::run_render_all_domain_no_declared_only_rows(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainNoNotBenchmarkReadyRows(args) => {
                        crate::commands::benchmark::readiness::all_domain_no_not_benchmark_ready_rows::run_render_all_domain_no_not_benchmark_ready_rows(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainNoPlannedRows(args) => {
                        crate::commands::benchmark::readiness::all_domain_no_planned_rows::run_render_all_domain_no_planned_rows(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainRetainedTools(args) => {
                        crate::commands::benchmark::readiness::all_domain_retained_tools::run_render_all_domain_retained_tools(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageToolAliasCheck(args) => {
                        crate::commands::benchmark::readiness::stage_tool_alias_check::run_render_stage_tool_alias_check(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderRemovedFromScope(args) => {
                        crate::commands::benchmark::readiness::removed_from_scope::run_render_removed_from_scope(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderAllDomainStageToolTable(args) => {
                        crate::commands::benchmark::readiness::all_domain_stage_tool_table::run_render_all_domain_stage_tool_table(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfComparableMetrics(args) => {
                        crate::commands::benchmark::readiness::vcf_comparable_metrics::run_render_vcf_comparable_metrics(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfExpectedBenchmarkResults(args) => {
                        crate::commands::benchmark::readiness::vcf_expected_benchmark_results::run_render_vcf_expected_benchmark_results(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfMissingResultReport(args) => {
                        crate::commands::benchmark::readiness::vcf_missing_result_report::run_render_vcf_missing_result_report(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfReportMap(args) => {
                        crate::commands::benchmark::readiness::vcf_report_map::run_render_vcf_report_map(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfParsersReportReady(args) => {
                        crate::commands::benchmark::readiness::vcf_parsers_report_ready::run_render_vcf_parsers_report_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfParserFixtureCoverage(args) => {
                        crate::commands::benchmark::readiness::vcf_parser_fixture_coverage::run_render_vcf_parser_fixture_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfNormalizedMetricsSchema(args) => {
                        crate::commands::benchmark::readiness::vcf_normalized_metrics_schema::run_render_vcf_normalized_metrics_schema(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfParserFailureTests(args) => {
                        crate::commands::benchmark::readiness::vcf_parser_failure_tests::run_render_vcf_parser_failure_tests(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAdapterMissingInputTests(args) => {
                        crate::commands::benchmark::readiness::vcf_adapter_missing_input_tests::run_render_vcf_adapter_missing_input_tests(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAdaptersReady(args) => {
                        crate::commands::benchmark::readiness::vcf_adapters_ready::run_render_vcf_adapters_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfActiveStageToolMatrix(args) => {
                        crate::commands::benchmark::readiness::vcf_active_stage_tool_matrix::run_render_vcf_active_stage_tool_matrix(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfLocalContainerSmoke(args) => {
                        crate::commands::benchmark::readiness::vcf_local_container_smoke::run_render_vcf_local_container_smoke(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfDamageFilterReady(args) => {
                        crate::commands::benchmark::readiness::vcf_damage_filter_ready::run_render_vcf_damage_filter_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfFilterReady(args) => {
                        crate::commands::benchmark::readiness::vcf_filter_ready::run_render_vcf_filter_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfGlPropagationReady(args) => {
                        crate::commands::benchmark::readiness::vcf_gl_propagation_ready::run_render_vcf_gl_propagation_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfCallGlReady(args) => {
                        crate::commands::benchmark::readiness::vcf_call_gl_ready::run_render_vcf_call_gl_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfCallDiploidReady(args) => {
                        crate::commands::benchmark::readiness::vcf_call_diploid_ready::run_render_vcf_call_diploid_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfCallPseudohaploidReady(args) => {
                        crate::commands::benchmark::readiness::vcf_call_pseudohaploid_ready::run_render_vcf_call_pseudohaploid_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAdmixtureReady(args) => {
                        crate::commands::benchmark::readiness::vcf_admixture_ready::run_render_vcf_admixture_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfPopulationStructureReady(args) => {
                        crate::commands::benchmark::readiness::vcf_population_structure_ready::run_render_vcf_population_structure_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAllRetainedToolsComplete(args) => {
                        crate::commands::benchmark::readiness::vcf_all_retained_tools_complete::run_render_vcf_all_retained_tools_complete(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfPcaReady(args) => {
                        crate::commands::benchmark::readiness::vcf_pca_ready::run_render_vcf_pca_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfImputationMetricsReady(args) => {
                        crate::commands::benchmark::readiness::vcf_imputation_metrics_ready::run_render_vcf_imputation_metrics_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfStatsReady(args) => {
                        crate::commands::benchmark::readiness::vcf_stats_ready::run_render_vcf_stats_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfQcReady(args) => {
                        crate::commands::benchmark::readiness::vcf_qc_ready::run_render_vcf_qc_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfPrepareReferencePanelReady(args) => {
                        crate::commands::benchmark::readiness::vcf_prepare_reference_panel_ready::run_render_vcf_prepare_reference_panel_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfCallReady(args) => {
                        crate::commands::benchmark::readiness::vcf_call_ready::run_render_vcf_call_ready(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAdapterOutputCoverage(args) => {
                        crate::commands::benchmark::readiness::vcf_adapter_output_coverage::run_render_vcf_adapter_output_coverage(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfCommands(args) => {
                        crate::commands::benchmark::readiness::vcf_rendered_commands::run_render_vcf_commands(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfAngsdAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_angsd_adapter::run_render_vcf_angsd_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfDescentFamilyAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_descent_family_adapter::run_render_vcf_descent_family_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfEigensoftAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_eigensoft_adapter::run_render_vcf_eigensoft_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfShapeit5Adapter(args) => {
                        crate::commands::benchmark::readiness::vcf_phasing_family_adapter::run_render_vcf_shapeit5_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfEagleAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_phasing_family_adapter::run_render_vcf_eagle_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfBeagleAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_phasing_family_adapter::run_render_vcf_beagle_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfImputationFamilyAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_imputation_family_adapter::run_render_vcf_imputation_family_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfPlinkAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_plink_family_adapter::run_render_vcf_plink_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfPlink2Adapter(args) => {
                        crate::commands::benchmark::readiness::vcf_plink_family_adapter::run_render_vcf_plink2_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfBcftoolsAdapter(args) => {
                        crate::commands::benchmark::readiness::vcf_bcftools_adapter::run_render_vcf_bcftools_adapter(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderMissingBenchmarkPairs(args) => {
                        crate::commands::benchmark::readiness::missing_benchmark_pairs::run_render_missing_benchmark_pairs(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderStageRegistryExtraPairs(args) => {
                        crate::commands::benchmark::readiness::stage_registry_extra_pairs::run_render_stage_registry_extra_pairs(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::ValidateToolExecutionModes(args) => {
                        crate::commands::benchmark::readiness::tool_execution_modes::run_validate_tool_execution_modes(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderToolIdNormalization(args) => {
                        crate::commands::benchmark::readiness::tool_id_normalization::run_render_tool_id_normalization(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::ValidateToolFamilies(args) => {
                        crate::commands::benchmark::readiness::tool_families::run_validate_tool_families(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderUnregisteredBenchmarkPairs(args) => {
                        crate::commands::benchmark::readiness::unregistered_benchmark_pairs::run_render_unregistered_benchmark_pairs(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderOrphanTools(args) => {
                        crate::commands::benchmark::readiness::orphan_tools::run_render_orphan_tools(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfOrphanTools(args) => {
                        crate::commands::benchmark::readiness::vcf_orphan_tools::run_render_vcf_orphan_tools(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfMatrixRegistryConsistency(args) => {
                        crate::commands::benchmark::readiness::vcf_matrix_registry_consistency::run_render_vcf_matrix_registry_consistency(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderVcfUndercoveredStages(args) => {
                        crate::commands::benchmark::readiness::vcf_undercovered_stages::run_render_vcf_undercovered_stages(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderUndercoveredStages(args) => {
                        crate::commands::benchmark::readiness::undercovered_stages::run_render_undercovered_stages(
                            args,
                        )?;
                    }
                },
                BenchCommand::Local { command } => match command {
                    cli::BenchLocalCommand::ListStages(args) => {
                        let cwd = std::env::current_dir()?;
                        let selected_domains = [
                            (
                                cli::BenchLocalStageListDomainArg::Fastq,
                                crate::commands::benchmark::local_stage_inventory::BenchLocalDomain::Fastq,
                            ),
                            (
                                cli::BenchLocalStageListDomainArg::Bam,
                                crate::commands::benchmark::local_stage_inventory::BenchLocalDomain::Bam,
                            ),
                            (
                                cli::BenchLocalStageListDomainArg::Vcf,
                                crate::commands::benchmark::local_stage_inventory::BenchLocalDomain::Vcf,
                            ),
                        ]
                        .into_iter()
                        .filter_map(|(requested_domain, inventory_domain)| {
                            args.domain.contains(&requested_domain).then_some(inventory_domain)
                        })
                        .collect::<Vec<_>>();

                        if selected_domains.len() == 1 {
                            let inventory = crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory(
                                &cwd,
                                selected_domains[0],
                            )?;
                            if cli.json || args.json {
                                render::json::print_pretty(&inventory)?;
                            } else {
                                println!(
                                    "{} local benchmark stages ({})",
                                    inventory.domain, inventory.stage_count
                                );
                                for stage in &inventory.stages {
                                    println!(
                                        "{}\t{}",
                                        stage.stage_id,
                                        stage.readiness_kind.as_str()
                                    );
                                }
                            }
                        } else {
                            let report = crate::commands::benchmark::local_stage_inventory::render_all_domain_stage_inventory(
                                &cwd,
                                &selected_domains,
                                std::path::PathBuf::from(
                                    crate::commands::benchmark::local_stage_inventory::DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
                                ),
                            )?;
                            if cli.json || args.json {
                                render::json::print_pretty(&report)?;
                            } else {
                                println!("{}", report.output_path);
                            }
                        }
                    }
                    cli::BenchLocalCommand::RenderVcfStageCatalog(args) => {
                        crate::commands::benchmark::local_vcf_stage_catalog::run_render_vcf_stage_catalog(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderVcfStageMatrix(args) => {
                        crate::commands::benchmark::local_vcf_stage_matrix::run_render_vcf_stage_matrix(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderVcfSmokeRoot(args) => {
                        crate::commands::benchmark::local_vcf_smoke_root::run_render_vcf_smoke_root(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfCallSmoke(args) => {
                        crate::commands::benchmark::local_vcf_call_smoke::run_vcf_call_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfCallDiploidSmoke(args) => {
                        crate::commands::benchmark::local_vcf_call_diploid_smoke::run_vcf_call_diploid_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfCallGlSmoke(args) => {
                        crate::commands::benchmark::local_vcf_call_gl_smoke::run_vcf_call_gl_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfDamageFilterSmoke(args) => {
                        crate::commands::benchmark::local_vcf_damage_filter_smoke::run_vcf_damage_filter_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfFilterSmoke(args) => {
                        crate::commands::benchmark::local_vcf_filter_smoke::run_vcf_filter_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfQcSmoke(args) => {
                        crate::commands::benchmark::local_vcf_qc_smoke::run_vcf_qc_smoke(&args)?;
                    }
                    cli::BenchLocalCommand::RunVcfAdmixtureSmoke(args) => {
                        crate::commands::benchmark::local_vcf_admixture_smoke::run_vcf_admixture_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfPopulationStructureSmoke(args) => {
                        crate::commands::benchmark::local_vcf_population_structure_smoke::run_vcf_population_structure_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfIbdSmoke(args) => {
                        crate::commands::benchmark::local_vcf_ibd_smoke::run_vcf_ibd_smoke(&args)?;
                    }
                    cli::BenchLocalCommand::RunVcfDemographySmoke(args) => {
                        crate::commands::benchmark::local_vcf_demography_smoke::run_vcf_demography_smoke(&args)?;
                    }
                    cli::BenchLocalCommand::RunVcfRohSmoke(args) => {
                        crate::commands::benchmark::local_vcf_roh_smoke::run_vcf_roh_smoke(&args)?;
                    }
                    cli::BenchLocalCommand::RunVcfPcaSmoke(args) => {
                        crate::commands::benchmark::local_vcf_pca_smoke::run_vcf_pca_smoke(&args)?;
                    }
                    cli::BenchLocalCommand::RunVcfStatsSmoke(args) => {
                        crate::commands::benchmark::local_vcf_stats_smoke::run_vcf_stats_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfGlPropagationSmoke(args) => {
                        crate::commands::benchmark::local_vcf_gl_propagation_smoke::run_vcf_gl_propagation_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfImputeSmoke(args) => {
                        crate::commands::benchmark::local_vcf_impute_smoke::run_vcf_impute_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfImputationMetricsSmoke(args) => {
                        crate::commands::benchmark::local_vcf_imputation_metrics_smoke::run_vcf_imputation_metrics_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfCallPseudohaploidSmoke(args) => {
                        crate::commands::benchmark::local_vcf_call_pseudohaploid_smoke::run_vcf_call_pseudohaploid_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfPhasingSmoke(args) => {
                        crate::commands::benchmark::local_vcf_phasing_smoke::run_vcf_phasing_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunVcfPrepareReferencePanelSmoke(args) => {
                        crate::commands::benchmark::local_vcf_prepare_reference_panel_smoke::run_vcf_prepare_reference_panel_smoke(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateVcfNoEmptyOutput(args) => {
                        crate::commands::benchmark::local_vcf_no_empty_output::run_validate_vcf_no_empty_output(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateVcfStageCatalogReady(args) => {
                        crate::commands::benchmark::local_vcf_stage_catalog_ready::run_validate_vcf_stage_catalog_ready(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateVcfSmokeSuiteReady(args) => {
                        crate::commands::benchmark::local_vcf_smoke_suite_ready::run_validate_vcf_smoke_suite_ready(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateVcfReferenceCompatibility(args) => {
                        crate::commands::benchmark::local_vcf_reference_compatibility::run_validate_vcf_reference_compatibility(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateVcfSampleCompatibility(args) => {
                        crate::commands::benchmark::local_vcf_sample_compatibility::run_validate_vcf_sample_compatibility(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateHpcSubmissionReady(args) => {
                        crate::commands::benchmark::local_hpc_submission_ready::run_validate_hpc_submission_ready(
                            args.output.clone(),
                            args.json,
                        )?;
                    }
                    cli::BenchLocalCommand::SimulateDagWatchdog(args) => {
                        crate::commands::benchmark::local_dag_watchdog_simulation::run_simulate_dag_watchdog(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidatePipelineDag(args) => {
                        crate::commands::benchmark::local_pipeline_dag::run_validate_pipeline_dag(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateCorpusFixture(args) => {
                        crate::commands::benchmark::local_corpus_fixture::run_validate_corpus_fixture(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateCorpusStageCompatibility(args) => {
                        crate::commands::benchmark::local_corpus_stage_compatibility::run_validate_corpus_stage_compatibility(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderCorpusSkipReport(args) => {
                        crate::commands::benchmark::local_corpus_skip_report::run_render_corpus_skip_report(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateTaxonomyDatabaseFixture(args) => {
                        crate::commands::benchmark::local_taxonomy_database_fixture::run_validate_taxonomy_database_fixture(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::JudgeTaxonomyOutput(args) => {
                        crate::commands::benchmark::local_taxonomy_output_judgment::run_judge_taxonomy_output(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateSlurmShellSyntax(args) => {
                        crate::commands::benchmark::local_slurm_shell_syntax::run_validate_slurm_shell_syntax(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateAllDomainSlurmShellSyntax(args) => {
                        crate::commands::benchmark::local_all_domain_slurm_shell_syntax::run_validate_all_domain_slurm_shell_syntax(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateSlurmDependencies(args) => {
                        crate::commands::benchmark::local_slurm_dependency_check::run_validate_slurm_dependencies(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateSlurmScriptBodies(args) => {
                        crate::commands::benchmark::local_slurm_script_bodies::run_validate_slurm_script_bodies(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateAllDomainSlurmScriptBodies(args) => {
                        crate::commands::benchmark::local_all_domain_slurm_script_bodies::run_validate_all_domain_slurm_script_bodies(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateAllDomainSlurmResultPaths(args) => {
                        crate::commands::benchmark::local_all_domain_slurm_path_convention::run_validate_all_domain_slurm_result_paths(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderAllDomainSlurmSubmitManifest(args) => {
                        crate::commands::benchmark::local_all_domain_slurm_submit_manifest::run_render_all_domain_slurm_submit_manifest(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderSlurmSubmitManifest(args) => {
                        crate::commands::benchmark::local_slurm_submit_manifest::run_render_slurm_submit_manifest(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderBenchmarkSummary(args) => {
                        crate::commands::benchmark::local_benchmark_summary::run_render_benchmark_summary(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::CheckManifestCompletion(args) => {
                        crate::commands::benchmark::local_stage_manifest_completion::run_check_manifest_completion(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::CheckOutputCompletion(args) => {
                        crate::commands::benchmark::local_stage_output_completion::run_check_output_completion(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::CollectRuntimeMetrics(args) => {
                        crate::commands::benchmark::local_stage_runtime_metrics::run_collect_runtime_metrics(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderToolComparisonTemplate(args) => {
                        crate::commands::benchmark::local_tool_comparison_template::run_render_tool_comparison_template(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ValidateStageResult(args) => {
                        crate::commands::benchmark::local_stage_result_manifest::run_validate_stage_result(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::MaterializeStage(args) => {
                        crate::commands::benchmark::local_stage_commands::run_materialize_stage(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::FakeRunEssentialPipelines(args) => {
                        crate::commands::benchmark::local_essential_pipeline_fake_runs::run_fake_run_essential_pipelines(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RunRealSmokeCoreSubset(args) => {
                        crate::commands::benchmark::local_real_smoke_core_subset::run_real_smoke_core_subset(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::FakeRunAllDomains(args) => {
                        crate::commands::benchmark::local_all_domain_fake_runs::run_fake_run_all_domains(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::FakeRunAllDomainFailures(args) => {
                        crate::commands::benchmark::local_all_domain_fake_failures::run_fake_run_all_domain_failures(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ExecuteAllDomainBenchmarkResult(args) => {
                        crate::commands::benchmark::local_all_domain_job_execution::run_execute_all_domain_benchmark_result(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::ExecuteEssentialPipelineNode(args) => {
                        crate::commands::benchmark::local_all_domain_job_execution::run_execute_essential_pipeline_node(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderAllDomainSlurmScripts(args) => {
                        crate::commands::benchmark::local_all_domain_slurm_scripts::run_render_all_domain_slurm_scripts(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::FakeRunFailures(args) => {
                        crate::commands::benchmark::local_stage_fake_runs::run_fake_run_failures(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::FakeRunStages(args) => {
                        crate::commands::benchmark::local_stage_fake_runs::run_fake_run_stages(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderSlurmScripts(args) => {
                        crate::commands::benchmark::local_slurm_dry_run::run_render_slurm_scripts(
                            &args,
                        )?;
                    }
                    cli::BenchLocalCommand::RenderStageCommands(args) => {
                        crate::commands::benchmark::local_stage_commands::run_render_stage_commands(
                            &args,
                        )?;
                    }
                },
                BenchCommand::CorpusFastq(args) => {
                    crate::commands::benchmark_corpus_fastq::run_benchmark_corpus_fastq(cli, args)?;
                }
                BenchCommand::Fastq { command } => match command {
                    BenchFastqCommand::Trim(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_trim(args)?;
                        let outcome = bench_fastq_trim(&catalog, &platform, None, &bench_args)?;
                        write_trim_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::TrimPolygTails(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_trim_polyg(args)?;
                        let outcome =
                            bench_fastq_trim_polyg_tails(&catalog, &platform, None, &bench_args)?;
                        write_trim_polyg_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::TrimTerminalDamage(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_trim_terminal_damage(args)?;
                        let outcome = bench_fastq_trim_terminal_damage(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_trim_terminal_damage_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Validate(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_validate(args)?;
                        let outcome =
                            bench_fastq_validate_reads(&catalog, &platform, None, &bench_args)?;
                        let qc_class = qc_class_label("fastq.validate_reads");
                        write_validate_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            qc_class,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::DetectAdapters(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_detect_adapters(args)?;
                        let outcome =
                            bench_fastq_detect_adapters(&catalog, &platform, None, &bench_args)?;
                        write_detect_adapters_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Filter(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_filter(args)?;
                        let outcome = bench_fastq_filter(&catalog, &platform, None, &bench_args)?;
                        write_filter_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::FilterLowComplexity(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_filter_low_complexity(args)?;
                        let outcome = bench_fastq_filter_low_complexity(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_filter_low_complexity_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Merge(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_merge(args)?;
                        let outcome = bench_fastq_merge(&catalog, &platform, None, &bench_args)?;
                        write_merge_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::ProfileReadLengths(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_profile_read_lengths(args)?;
                        let outcome = bench_fastq_profile_read_lengths(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_read_lengths_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::RemoveDuplicates(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_remove_duplicates(args)?;
                        let outcome =
                            bench_fastq_remove_duplicates(&catalog, &platform, None, &bench_args)?;
                        write_duplicates_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::RemoveChimeras(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_remove_chimeras(args)?;
                        let outcome =
                            bench_fastq_remove_chimeras(&catalog, &platform, None, &bench_args)?;
                        write_chimeras_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::NormalizePrimers(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_normalize_primers(args)?;
                        let outcome =
                            bench_fastq_normalize_primers(&catalog, &platform, None, &bench_args)?;
                        write_normalize_primers_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::InferAsvs(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_infer_asvs(args)?;
                        let outcome =
                            bench_fastq_infer_asvs(&catalog, &platform, None, &bench_args)?;
                        write_infer_asvs_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::ClusterOtus(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_cluster_otus(args)?;
                        let outcome =
                            bench_fastq_cluster_otus(&catalog, &platform, None, &bench_args)?;
                        write_cluster_otus_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::NormalizeAbundance(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_normalize_abundance(args)?;
                        let outcome = bench_fastq_normalize_abundance(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_normalize_abundance_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Stats(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_stats(args)?;
                        let outcome =
                            bench_fastq_stats_neutral(&catalog, &platform, None, &bench_args)?;
                        write_stats_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::ProfileOverrepresentedSequences(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_profile_overrepresented(args)?;
                        let outcome = bench_fastq_profile_overrepresented(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_overrepresented_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Correct(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_correct(args)?;
                        let outcome = bench_fastq_correct(&catalog, &platform, None, &bench_args)?;
                        write_correct_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::ReportQc(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_qc_post(args)?;
                        let outcome = bench_fastq_qc_post(&catalog, &platform, None, &bench_args)?;
                        write_qc_post_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Umi(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_umi(args)?;
                        let outcome = bench_fastq_umi(&catalog, &platform, None, &bench_args)?;
                        write_umi_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::IndexReference(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_index_reference(args)?;
                        let outcome =
                            bench_fastq_index_reference(&catalog, &platform, None, &bench_args)?;
                        write_index_reference_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Screen(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_screen(args)?;
                        let outcome = bench_fastq_screen(&catalog, &platform, None, &bench_args)?;
                        write_screen_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::DepleteHost(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_deplete_host(args)?;
                        let outcome =
                            bench_fastq_deplete_host(&catalog, &platform, None, &bench_args)?;
                        write_deplete_host_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::DepleteReferenceContaminants(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_deplete_reference_contaminants(args)?;
                        let outcome = bench_fastq_deplete_reference_contaminants(
                            &catalog,
                            &platform,
                            None,
                            &bench_args,
                        )?;
                        write_deplete_reference_contaminants_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::DepleteRrna(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_deplete_rrna(args)?;
                        let outcome =
                            bench_fastq_deplete_rrna(&catalog, &platform, None, &bench_args)?;
                        write_deplete_rrna_report(
                            &outcome.bench_dir,
                            &outcome.records,
                            &outcome.failures,
                            outcome.explain,
                        )?;
                        if !outcome.failures.is_empty() {
                            return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
                        }
                    }
                    BenchFastqCommand::Preprocess(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        bench_fastq_preprocess(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_preprocess(args),
                        )?;
                    }
                },
                BenchCommand::Bam { command } => match command {
                    BenchBamCommand::Stage(args) => {
                        let registry = load_manifests(registry_path)
                            .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
                        bijux_dna_api::v1::api::bench::bench_bam_stage(
                            &bench_bam_stage_args_to_api(args),
                            &registry,
                            cli.platform.as_deref(),
                        )?;
                    }
                    BenchBamCommand::Pipeline(args) => {
                        let registry = load_manifests(registry_path)
                            .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
                        bijux_dna_api::v1::api::bench::bench_bam_pipeline(
                            &bench_bam_pipeline_args_to_api(args),
                            &registry,
                            cli.platform.as_deref(),
                        )?;
                    }
                },
                BenchCommand::Schema { stage } => {
                    print_bench_schema(stage)?;
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
