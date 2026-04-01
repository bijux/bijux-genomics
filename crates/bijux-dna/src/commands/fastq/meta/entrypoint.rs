use super::analyze::handle_analyze_command;
use super::debug::handle_debug_command;
use super::pipelines::handle_pipelines_command;
use crate::cli::BenchConfigCommand;
use crate::commands::fastq::api_bridge::{
    bench_bam_pipeline_args_to_api, bench_bam_stage_args_to_api, resolve_profile_alias,
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
    bench_fastq_trim_terminal_damage, bench_fastq_umi, bench_fastq_validate_reads, cli, env_doctor,
    load_image_catalog, load_platform, print_bench_schema, print_env_export_json, print_env_images,
    print_env_info, print_env_registry_list, qc_class_label, render, run_env_prep, run_env_smoke,
    run_env_smoke_for_stage, run_image_qa, set_tool_tier_policy, workspace_audit,
    write_chimeras_report, write_cluster_otus_report, write_correct_report,
    write_deplete_host_report, write_deplete_reference_contaminants_report,
    write_deplete_rrna_report, write_detect_adapters_report, write_duplicates_report,
    write_filter_low_complexity_report, write_filter_report, write_index_reference_report,
    write_infer_asvs_report, write_merge_report, write_normalize_abundance_report,
    write_normalize_primers_report, write_overrepresented_report, write_qc_post_report,
    write_read_lengths_report, write_run_report_from_facts, write_run_summary_from_facts,
    write_screen_report, write_stage_summary_csv, write_stats_report, write_trim_polyg_report,
    write_trim_report, write_trim_terminal_damage_report, write_umi_report, write_validate_report,
    BenchBamCommand, BenchCommand, BenchFastqCommand, Cli, DnaCommand, EnvCommand, Objective, Path,
    PoliciesCommand, Result,
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
        DnaCommand::Environment(args) => {
            match &args.command {
                EnvCommand::List => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    print_env_registry_list(&registry_path)?;
                }
                EnvCommand::ExportJson => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    print_env_export_json(&registry_path)?;
                }
                EnvCommand::ExportContainers { json } => {
                    if !json {
                        return Err(anyhow!("environment export-containers requires --json"));
                    }
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    crate::commands::cli::env::print_registry_export_containers_json(
                        &registry_path,
                    )?;
                }
                EnvCommand::ExportHpc { json, hpc_root } => {
                    let root = hpc_root.clone().map_or_else(
                        || {
                            crate::commands::hpc::load_hpc_config()
                                .map(|cfg| cfg.resolve_paths().root)
                        },
                        Ok,
                    )?;
                    let layout = crate::commands::hpc::HpcLayout::from_root(&root);
                    let export = crate::commands::hpc::export_hpc_env_json(&layout)?;
                    if *json {
                        render::json::print_pretty(&export)?;
                    } else {
                        println!("containers_dir={}", export.containers_dir);
                        println!("sif_count={}", export.sifs.len());
                    }
                }
                EnvCommand::EnsureImages(args) => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    let hpc_root = args.hpc_root.clone().map_or_else(
                        || {
                            crate::commands::hpc::load_hpc_config()
                                .map(|cfg| cfg.resolve_paths().root)
                        },
                        Ok,
                    )?;
                    let stages = match (&args.stage, &args.stages) {
                        (Some(stage), None) => stage.clone(),
                        (None, Some(stages)) => stages.clone(),
                        _ => {
                            return Err(anyhow!(
                                "environment ensure-images requires exactly one of --stage or --stages"
                            ));
                        }
                    };
                    let report = crate::commands::cli::env::ensure_apptainer_images(
                        &registry_path,
                        &hpc_root,
                        &args.domain,
                        &stages,
                        args.force_smoke,
                        args.repair_mismatch,
                    )?;
                    if args.json {
                        render::json::print_pretty(&report)?;
                    } else {
                        println!("schema_version={}", report.schema_version);
                        println!("requested_tools={}", report.tools.len());
                        println!("built={}", report.built);
                        println!("reused={}", report.reused);
                        println!("quick_smoked={}", report.quick_smoked);
                        println!("failed={}", report.failed);
                    }
                }
                EnvCommand::LintApptainerDefs => {
                    let cwd = std::env::current_dir()?;
                    crate::commands::cli::env::lint_apptainer_defs(&cwd)?;
                }
                EnvCommand::SifInventory { hpc_root, json } => {
                    let root = hpc_root.clone().map_or_else(
                        || {
                            crate::commands::hpc::load_hpc_config()
                                .map(|cfg| cfg.resolve_paths().root)
                        },
                        Ok,
                    )?;
                    let report = crate::commands::cli::env::sif_inventory(&root)?;
                    if *json {
                        render::json::print_pretty(&report)?;
                    } else {
                        println!("containers_dir={}", report.containers_dir);
                        println!("sif_count={}", report.entries.len());
                    }
                }
                EnvCommand::Ensure(args) => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    let domain = crate::commands::cli::env::parse_stage_domain(&args.stage)?;
                    let hpc_root = args.hpc_root.clone().map_or_else(
                        || {
                            crate::commands::hpc::load_hpc_config()
                                .map(|cfg| cfg.resolve_paths().root)
                        },
                        Ok,
                    )?;
                    let report = crate::commands::cli::env::ensure_apptainer_images(
                        &registry_path,
                        &hpc_root,
                        &domain,
                        &args.stage,
                        args.force_smoke,
                        args.repair_mismatch,
                    )?;
                    if args.json {
                        render::json::print_pretty(&report)?;
                    } else {
                        println!("schema_version={}", report.schema_version);
                        println!("requested_tools={}", report.tools.len());
                        println!("built={}", report.built);
                        println!("reused={}", report.reused);
                        println!("quick_smoked={}", report.quick_smoked);
                        println!("failed={}", report.failed);
                    }
                }
                EnvCommand::ApptainerQaMatrix { hpc_root, out } => {
                    let root = hpc_root.clone().map_or_else(
                        || {
                            crate::commands::hpc::load_hpc_config()
                                .map(|cfg| cfg.resolve_paths().root)
                        },
                        Ok,
                    )?;
                    let markdown =
                        crate::commands::cli::env::generate_apptainer_qa_matrix_markdown(&root)?;
                    if let Some(parent) = out.parent() {
                        bijux_dna_infra::ensure_dir(parent)?;
                    }
                    bijux_dna_api::v1::api::run::atomic_write_bytes(out, markdown.as_bytes())?;
                    println!("qa_matrix={}", out.display());
                }
                EnvCommand::Smoke(args) => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    if let Some(stage) = args.stage.as_deref() {
                        run_env_smoke_for_stage(&registry_path, &args.runtime, stage)?;
                    } else if let Some(tool) = args.tool.as_deref() {
                        run_env_smoke(&args.runtime, tool)?;
                    } else {
                        return Err(anyhow!(
                            "environment smoke requires either <tool> or --stage"
                        ));
                    }
                }
                EnvCommand::Prep(args) => {
                    let cwd = std::env::current_dir()?;
                    let registry_path =
                        bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    run_env_prep(
                        &registry_path,
                        &args.runtime,
                        args.tool.as_deref(),
                        args.stage.as_deref(),
                    )?;
                }
                EnvCommand::Images => {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog = load_image_catalog()
                        .map_err(|err| anyhow!("failed to load images: {err}"))?;
                    print_env_images(&catalog, &platform)?;
                }
                EnvCommand::Info => {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog = load_image_catalog()
                        .map_err(|err| anyhow!("failed to load images: {err}"))?;
                    print_env_info(&catalog, &platform);
                }
                EnvCommand::Doctor => {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog = load_image_catalog()
                        .map_err(|err| anyhow!("failed to load images: {err}"))?;
                    env_doctor(&catalog, &platform);
                }
            }
            Ok(true)
        }
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
