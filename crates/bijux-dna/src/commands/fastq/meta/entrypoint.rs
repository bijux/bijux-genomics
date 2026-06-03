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
                    cli::BenchReadinessCommand::RenderBamStageDecisionTable(args) => {
                        crate::commands::benchmark::readiness::bam_stage_decision_table::run_render_bam_stage_decision_table(
                            args,
                        )?;
                    }
                    cli::BenchReadinessCommand::RenderBamCommandAdapterCoverage(args) => {
                        crate::commands::benchmark::readiness::bam_command_adapter_coverage::run_render_bam_command_adapter_coverage(
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
                    cli::BenchReadinessCommand::RenderUndercoveredStages(args) => {
                        crate::commands::benchmark::readiness::undercovered_stages::run_render_undercovered_stages(
                            args,
                        )?;
                    }
                },
                BenchCommand::Local { command } => match command {
                    cli::BenchLocalCommand::ListStages(args) => {
                        let cwd = std::env::current_dir()?;
                        let domain = match args.domain {
                            cli::BenchLocalDomainArg::Fastq => {
                                crate::commands::benchmark::local_stage_inventory::BenchLocalDomain::Fastq
                            }
                            cli::BenchLocalDomainArg::Bam => {
                                crate::commands::benchmark::local_stage_inventory::BenchLocalDomain::Bam
                            }
                        };
                        let inventory = crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory(
                            &cwd,
                            domain,
                        )?;
                        if cli.json || args.json {
                            render::json::print_pretty(&inventory)?;
                        } else {
                            println!(
                                "{} local benchmark stages ({})",
                                inventory.domain, inventory.stage_count
                            );
                            for stage in &inventory.stages {
                                println!("{}\t{}", stage.stage_id, stage.readiness_kind.as_str());
                            }
                        }
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
                    cli::BenchLocalCommand::ValidateSlurmShellSyntax(args) => {
                        crate::commands::benchmark::local_slurm_shell_syntax::run_validate_slurm_shell_syntax(
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
