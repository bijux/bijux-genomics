#[allow(clippy::too_many_lines)]
pub(crate) fn handle_meta_commands(cli: &Cli, domain_dir: &Path) -> Result<bool> {
    match &cli.command {
        Commands::ValidateManifests => {
            let registry = load_manifests(domain_dir)
                .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
            println!(
                "validated {} stages and {} tools",
                registry.stages().len(),
                registry
                    .stages()
                    .keys()
                    .map(|stage| registry.tools_for_stage(stage).len())
                    .sum::<usize>()
            );
            Ok(true)
        }
        Commands::Platform => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            render::json::print_pretty(&platform)?;
            Ok(true)
        }
        Commands::ImageQa => {
            run_image_qa(cli.platform.as_deref())?;
            Ok(true)
        }
        Commands::Replay(args) => {
            if let Some(manifest_path) = args.manifest.as_ref() {
                bijux_api::v1::api::run::replay_manifest(manifest_path, args.verify_only)?;
                return Ok(true);
            }
            let manifest_path = args
                .search_root
                .join(&args.run_id)
                .join("run_manifest.json");
            bijux_api::v1::api::run::replay_manifest(&manifest_path, args.verify_only)?;
            Ok(true)
        }
        Commands::Compare(args) => {
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = if let Some(baseline) = args.baseline.as_ref() {
                let baseline_dir = args.search_root.join(baseline);
                compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
            } else {
                compare_runs(&run_a, &run_b, &objective)?
            };
            let output_dir = args.output_dir.as_ref().unwrap_or(&args.search_root);
            bijux_api::v1::api::run::ensure_dir(output_dir)?;
            let path = output_dir.join("compare.json");
            atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                .map_err(anyhow::Error::from)?;
            render::json::print_pretty(&result)?;
            Ok(true)
        }
        Commands::Policies { command } => {
            match command {
                PoliciesCommand::Audit { out } => {
                    workspace_audit(out)?;
                }
            }
            Ok(true)
        }
        Commands::Pipelines { command } => match command {
            PipelinesCommand::List {
                domain,
                show_experimental,
            } => {
                let profiles = bijux_api::v1::api::plan::select_pipelines(
                    domain.map(cli::parse::PipelineDomainArg::as_domain),
                    *show_experimental,
                );
                for profile in profiles {
                    println!(
                        "{}\t{}\t{}",
                        profile.id.as_str(),
                        profile.stability.as_str(),
                        profile.description
                    );
                }
                Ok(true)
            }
            PipelinesCommand::Explain { id } => {
                let profile = bijux_api::v1::api::plan::select_pipelines(None, true)
                    .into_iter()
                    .find(|profile| profile.id.as_str() == id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {id}"))?;
                let payload = serde_json::json!({
                    "profile": profile,
                    "defaults_ledger": profile.defaults_ledger(),
                    "promised_outputs": profile.capabilities.produces_outputs,
                    "report_sections": profile.capabilities.report_sections,
                });
                render::json::print_pretty(&payload)?;
                Ok(true)
            }
            PipelinesCommand::Audit {
                domain,
                show_experimental,
            } => {
                let profiles = bijux_api::v1::api::plan::select_pipelines(
                    domain.map(cli::parse::PipelineDomainArg::as_domain),
                    *show_experimental,
                );
                for profile in profiles {
                    println!(
                        "{}\t{}\t{}",
                        profile.id.as_str(),
                        profile.stability.as_str(),
                        profile.description
                    );
                    let stage_ids = match profile.id.as_str() {
                        "fastq-to-fastq__default__v1" | "fastq-to-fastq__minimal__v1" => {
                            bijux_api::v1::api::plan::fastq_pipeline_stage_ids(profile.id.as_str())
                        }
                        "fastq-to-bam__default__v1" | "fastq-to-bam__adna_shotgun__v1" => {
                            bijux_api::v1::api::plan::cross_fastq_to_bam_stage_ids(
                                profile.id.as_str(),
                            )
                        }
                        "bam-to-bam__default__v1"
                        | "bam-to-bam__adna_shotgun__v1"
                        | "bam-to-bam__adna_capture__v1" => {
                            bijux_api::v1::api::plan::bam_pipeline_stage_ids(profile.id.as_str())
                        }
                        _ => Vec::new(),
                    };
                    for stage_id in stage_ids {
                        if stage_id.starts_with("bam.") {
                            let stage =
                                bijux_api::v1::api::bench::BamStage::try_from(stage_id.as_str())
                                    .map_err(|_| anyhow!("unknown BAM stage {stage_id}"))?;
                            let completeness =
                                bijux_api::v1::api::bench::bam_stage_completeness(stage);
                            println!(
                                    "  {stage_id}\tcomplete={}\targs={}\tartifacts={}\tparsers={}\tinvariants={}",
                                    completeness.is_complete(),
                                    completeness.has_args_builder,
                                    completeness.has_artifact_contract,
                                    completeness.has_parser_fixtures,
                                    completeness.has_invariants
                                );
                        } else {
                            println!("  {stage_id}\tcomplete=unknown");
                        }
                    }
                }
                Ok(true)
            }
        },
        Commands::Analyze { command } => {
            match command {
                AnalyzeCommand::Runs(args) => {
                    let query = bijux_api::v1::api::run::RunQuery {
                        stage: args.stage.clone(),
                        tool: args.tool.clone(),
                        objective: args.objective.map(|obj| obj.as_str().to_string()),
                        success: args.success,
                    };
                    let runs = bijux_api::v1::api::run::query_runs(&args.index, &query)?;
                    render::json::print_pretty(&runs)?;
                }
                AnalyzeCommand::Summary(args) => {
                    let run_dir = args.search_root.join(&args.run_id);
                    let summary_path = run_dir.join("run_summary.json");
                    if summary_path.exists() {
                        let summary = load_run_summary(&summary_path)?;
                        render::json::print_pretty(&summary)?;
                    } else {
                        let facts_path = run_dir.join("facts.jsonl");
                        let facts = load_facts_auto(&facts_path)?;
                        write_run_summary_from_facts(&summary_path, &facts)?;
                        let summary = load_run_summary(&summary_path)?;
                        render::json::print_pretty(&summary)?;
                    }
                }
                AnalyzeCommand::Compare(args) => {
                    let objective = objective_spec(args.objective.into());
                    let run_a = args.search_root.join(&args.run_a);
                    let run_b = args.search_root.join(&args.run_b);
                    let result = if let Some(baseline) = args.baseline.as_ref() {
                        let baseline_dir = args.search_root.join(baseline);
                        compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
                    } else {
                        compare_runs(&run_a, &run_b, &objective)?
                    };
                    let output_dir = args.output_dir.as_ref().unwrap_or(&args.search_root);
                    bijux_api::v1::api::run::ensure_dir(output_dir)?;
                    let path = output_dir.join("compare.json");
                    atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                        .map_err(anyhow::Error::from)?;
                    render::json::print_pretty(&result)?;
                }
                AnalyzeCommand::Rank(args) => {
                    let run_dir = args.search_root.join(&args.run_id);
                    let facts_path = run_dir.join("facts.jsonl");
                    let facts = load_facts_auto(&facts_path)?;
                    let mut by_tool: BTreeMap<String, Vec<&bijux_api::v1::api::run::FactsRowV1>> =
                        BTreeMap::new();
                    for row in facts.iter().filter(|row| row.stage_id == args.stage) {
                        by_tool.entry(row.tool_id.clone()).or_default().push(row);
                    }
                    let mut inputs = Vec::new();
                    for (tool, rows) in by_tool {
                        let denom = f64::from(u32::try_from(rows.len().max(1)).unwrap_or(u32::MAX));
                        let runtime = rows.iter().map(|row| row.runtime_s).sum::<f64>() / denom;
                        let memory = rows.iter().map(|row| row.memory_mb).sum::<f64>() / denom;
                        let read_retention =
                            rows.iter()
                                .find_map(|row| match (row.reads_in, row.reads_out) {
                                    #[allow(clippy::cast_precision_loss)]
                                    (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
                                    _ => None,
                                });
                        let base_retention =
                            rows.iter()
                                .find_map(|row| match (row.bases_in, row.bases_out) {
                                    #[allow(clippy::cast_precision_loss)]
                                    (Some(bi), Some(bo)) if bi > 0 => Some(bo as f64 / bi as f64),
                                    _ => None,
                                });
                        let error_reduction_proxy = rows.iter().find_map(|row| {
                            row.metrics
                                .get("mean_q_delta")
                                .and_then(serde_json::Value::as_f64)
                        });
                        inputs.push(RankInput {
                            tool,
                            runtime_s: runtime,
                            memory_mb: memory,
                            read_retention,
                            base_retention,
                            error_reduction_proxy,
                        });
                    }
                    let rankings = bijux_api::v1::api::bench::build_rankings(&inputs)?;
                    render::json::print_pretty(&rankings)?;
                }
                AnalyzeCommand::Report(args) => {
                    let (run_dir, facts_path) = resolve_report_inputs(args)?;
                    let facts = load_facts_auto(&facts_path)?;
                    let report_path = write_run_report_from_facts(&run_dir, &facts)?;
                    let summary_csv = run_dir.join("summary.csv");
                    write_stage_summary_csv(&summary_csv, &facts)?;
                    match args.format.as_str() {
                        "json" => {
                            let raw = std::fs::read_to_string(&report_path)?;
                            println!("{raw}");
                        }
                        "html" | "bundle" => {
                            let report_raw = std::fs::read_to_string(&report_path)?;
                            let report_json: serde_json::Value = serde_json::from_str(&report_raw)
                                .unwrap_or_else(|_| {
                                    serde_json::json!({
                                        "error": "failed to parse report.json"
                                    })
                                });
                            let index_html = render_report_bundle_html(&report_json);
                            let report_html = run_dir.join("report.html");
                            atomic_write_bytes(&report_html, index_html.as_bytes())
                                .map_err(anyhow::Error::from)?;
                            if args.format == "bundle" {
                                let bundle_dir = run_dir.join("report_bundle");
                                bijux_api::v1::api::run::ensure_dir(&bundle_dir)?;
                                atomic_write_bytes(
                                    &bundle_dir.join("index.html"),
                                    index_html.as_bytes(),
                                )
                                .map_err(anyhow::Error::from)?;
                                atomic_write_bytes(
                                    &bundle_dir.join("report.json"),
                                    report_raw.as_bytes(),
                                )
                                .map_err(anyhow::Error::from)?;
                                println!("report bundle written to {}", bundle_dir.display());
                            } else {
                                println!("report html written to {}", report_html.display());
                            }
                        }
                        _ => {
                            println!("report written to {}", report_path.display());
                        }
                    }
                }
            }
            Ok(true)
        }
        Commands::Env { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                EnvCommand::Images => {
                    print_env_images(&catalog, &platform)?;
                }
                EnvCommand::Info => {
                    print_env_info(&catalog, &platform);
                }
                EnvCommand::Doctor => {
                    env_doctor(&catalog, &platform);
                }
            }
            Ok(true)
        }
        Commands::Bench { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                BenchCommand::Fastq { command } => match command {
                    BenchFastqCommand::Trim(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome =
                            bench_fastq_trim(&catalog, &platform, None, &bench_args_trim(args))?;
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
                    BenchFastqCommand::Validate(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome = bench_fastq_validate_pre(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_validate(args),
                        )?;
                        let qc_class = qc_class_label("fastq.validate_pre");
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
                    BenchFastqCommand::Filter(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome = bench_fastq_filter(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_filter(args),
                        )?;
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
                    BenchFastqCommand::Merge(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome =
                            bench_fastq_merge(&catalog, &platform, None, &bench_args_merge(args))?;
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
                    BenchFastqCommand::Stats(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome = bench_fastq_stats_neutral(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_stats(args),
                        )?;
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
                    BenchFastqCommand::Correct(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome = bench_fastq_correct(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_correct(args),
                        )?;
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
                    BenchFastqCommand::QcPost(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let outcome = bench_fastq_qc_post(
                            &catalog,
                            &platform,
                            None,
                            &bench_args_qc_post(args),
                        )?;
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
                        let outcome =
                            bench_fastq_umi(&catalog, &platform, None, &bench_args_umi(args))?;
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
                    BenchFastqCommand::Screen(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        bench_fastq_screen(&catalog, &platform, None, &bench_args_screen(args))?;
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
                        let registry = load_manifests(domain_dir)
                            .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
                        bijux_api::v1::api::bench::bench_bam_stage(
                            &bench_bam_stage_args_to_api(args),
                            &registry,
                            cli.platform.as_deref(),
                        )?;
                    }
                    BenchBamCommand::Pipeline(args) => {
                        let registry = load_manifests(domain_dir)
                            .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
                        bijux_api::v1::api::bench::bench_bam_pipeline(
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
        Commands::Fastq { .. } | Commands::Bam { .. } => Ok(false),
    }
}

fn bench_bam_stage_args_to_api(
    args: &crate::commands::cli::parse::BenchBamStageArgs,
) -> bijux_api::v1::api::bench::BenchBamStageArgs {
    bijux_api::v1::api::bench::BenchBamStageArgs {
        sample_id: args.sample_id.clone(),
        stage: args.stage.stage(),
        bam: args.bam.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        allow_silver: args.allow_silver,
        allow_experimental: args.allow_experimental,
        replicates: args.replicates,
        jobs: args.jobs,
        dry_run: args.dry_run,
    }
}

fn bench_bam_pipeline_args_to_api(
    args: &crate::commands::cli::parse::BenchBamPipelineArgs,
) -> bijux_api::v1::api::bench::BenchBamPipelineArgs {
    bijux_api::v1::api::bench::BenchBamPipelineArgs {
        profile: args.profile.clone(),
        sample_id: args.sample_id.clone(),
        bam: args.bam.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        allow_silver: args.allow_silver,
        allow_experimental: args.allow_experimental,
        replicates: args.replicates,
        jobs: args.jobs,
        dry_run: args.dry_run,
    }
}
