#[allow(clippy::too_many_lines)]
fn handle_fastq_bench(
    cli: &Cli,
    registry: &bijux_api::v1::types::ToolRegistry,
) -> Result<bool> {
    let Commands::Fastq { command } = &cli.command else {
        return Ok(false);
    };

    let (allow_silver, allow_experimental) = tool_tier_policy_for_fastq(command);
    set_tool_tier_policy(allow_silver, allow_experimental);

    if let Some(done) = handle_fastq_discovery(command, registry)? {
        return Ok(done);
    }

    match command {
        FastqCommand::Doctor => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            env_doctor(&catalog, &platform);
            Ok(true)
        }
        FastqCommand::Trim(args) if is_bench_requested_trim(args) => {
            set_tool_tier_policy(args.common.allow_silver, args.common.allow_experimental);
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = bench_args_from_trim(args)?;
            let outcome = bench_fastq_trim(&catalog, &platform, runner, &bench_args)?;
            write_trim_report(
                &outcome.bench_dir,
                &outcome.records,
                &outcome.failures,
                outcome.explain,
            )?;
            if !outcome.failures.is_empty() {
                return Err(anyhow!("benchmark failures: {}", outcome.failures.len()));
            }
            Ok(true)
        }
        FastqCommand::ValidatePre(args) if is_bench_requested_validate(args) => {
            set_tool_tier_policy(args.common.allow_silver, args.common.allow_experimental);
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = bench_args_from_validate(args)?;
            let outcome = bench_fastq_validate_pre(&catalog, &platform, runner, &bench_args)?;
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
            Ok(true)
        }
        FastqCommand::Preprocess(args) => {
            set_tool_tier_policy(args.common.allow_silver, args.common.allow_experimental);
            set_scientific_preset(args.scientific_preset);
            if let Some(profile_id) = args.pipeline_profile.as_ref() {
                if let Ok(profile) =
                    bijux_api::v1::pipelines::select_pipeline(bijux_api::v1::pipelines::Domain::Cross, profile_id)
                {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog =
                        load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
                    let runner = cli::parse_runner_override(args.env.as_deref())?;
                    let bench_args = preprocess_args_from_cli(args)?;
                    let cross_args = fastq_cross_args_from_cli(args);
                    bijux_api::v1::run::run_fastq_to_bam_profile(
                        registry,
                        &catalog,
                        &platform,
                        runner,
                        &bench_args,
                        &cross_args,
                        &profile,
                    )?;
                    return Ok(true);
                }
            }
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.env.as_deref())?;
            let bench_args = preprocess_args_from_cli(args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Run(args) => {
            set_tool_tier_policy(
                args.args.common.allow_silver,
                args.args.common.allow_experimental,
            );
            set_scientific_preset(args.args.scientific_preset);
            if let Some(profile_id) = args.args.pipeline_profile.as_ref() {
                if let Ok(profile) =
                    bijux_api::v1::pipelines::select_pipeline(bijux_api::v1::pipelines::Domain::Cross, profile_id)
                {
                    let platform = load_platform(cli.platform.as_deref())
                        .map_err(|err| anyhow!("failed to load platform: {err}"))?;
                    let catalog =
                        load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
                    let runner = cli::parse_runner_override(args.args.env.as_deref())?;
                    let bench_args = preprocess_args_from_cli(&args.args)?;
                    let cross_args = fastq_cross_args_from_cli(&args.args);
                    bijux_api::v1::run::run_fastq_to_bam_profile(
                        registry,
                        &catalog,
                        &platform,
                        runner,
                        &bench_args,
                        &cross_args,
                        &profile,
                    )?;
                    return Ok(true);
                }
            }
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            let runner = cli::parse_runner_override(args.args.env.as_deref())?;
            let bench_args = preprocess_args_from_cli(&args.args)?;
            bench_fastq_preprocess(&catalog, &platform, runner, &bench_args)?;
            Ok(true)
        }
        FastqCommand::Benchmark(args) => {
            let stage_id = normalize_fastq_stage_id(&args.stage);
            let summary = benchmark_runs(&args.runs, &stage_id, args.objective.into())?;
            let (json_path, csv_path) = write_benchmark_exports(&args.runs, &summary)?;
            let html_path = args.runs.join(format!("benchmark_{}.html", summary.stage));
            println!("{}", serde_json::to_string_pretty(&summary)?);
            println!("benchmark_json: {}", json_path.display());
            println!("benchmark_csv: {}", csv_path.display());
            println!("benchmark_html: {}", html_path.display());
            Ok(true)
        }
        FastqCommand::Analyze(args) => {
            let stage_id = normalize_fastq_stage_id(&args.stage);
            let summary = benchmark_runs(&args.runs, &stage_id, args.objective.into())?;
            let (json_path, csv_path) = write_benchmark_exports(&args.runs, &summary)?;
            let html_path = args.runs.join(format!("benchmark_{}.html", summary.stage));
            println!("{}", serde_json::to_string_pretty(&summary)?);
            println!("benchmark_json: {}", json_path.display());
            println!("benchmark_csv: {}", csv_path.display());
            println!("benchmark_html: {}", html_path.display());
            Ok(true)
        }
        FastqCommand::Compare(args) => {
            let objective = objective_spec(Objective::Balanced);
            let run_a = args.search_root.join(&args.run_a);
            let run_b = args.search_root.join(&args.run_b);
            let result = if let Some(baseline) = args.baseline.as_ref() {
                let baseline_dir = args.search_root.join(baseline);
                compare_runs_with_baseline(&run_a, &run_b, &baseline_dir, &objective)?
            } else {
                compare_runs(&run_a, &run_b, &objective)?
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(true)
        }
        _ => {
            let (stage, _tool, common) = cli::resolve_stage_tool(&cli.command);
            if common.list_tools {
                list_fastq_tools(registry, &stage.0);
                return Ok(true);
            }
            Ok(false)
        }
    }
}
fn handle_fastq_discovery(
    command: &FastqCommand,
    registry: &bijux_api::v1::types::ToolRegistry,
) -> Result<Option<bool>> {
    match command {
        FastqCommand::ListStages => {
            list_fastq_stages();
            Ok(Some(true))
        }
        FastqCommand::Stages => {
            list_fastq_stage_registry();
            Ok(Some(true))
        }
        FastqCommand::ListTools { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            list_fastq_tools(registry, &stage_id);
            Ok(Some(true))
        }
        FastqCommand::Explain { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            explain_fastq_stage(registry, &stage_id)?;
            Ok(Some(true))
        }
        FastqCommand::Trim(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        FastqCommand::Preprocess(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
fn list_fastq_stages() {
    for stage in &bijux_api::v1::fastq::STAGES {
        println!("{}", stage.stage_id);
    }
    print_bank_presets();
}
fn list_fastq_stage_registry() {
    for stage in &bijux_api::v1::fastq::STAGES {
        println!("{}", stage.stage_id);
    }
    print_bank_presets();
}
fn print_bank_presets() {
    if let Ok(selection) = resolve_adapter_selection(None, None, None) {
        let mut presets: Vec<String> = selection
            .presets
            .presets
            .iter()
            .map(|preset| preset.name.clone())
            .collect();
        presets.sort();
        if !presets.is_empty() {
            println!("adapter_presets: {}", presets.join(", "));
        }
    }
    if let Ok(selection) = bijux_api::v1::fastq::fastq_banks::resolve_polyx_selection(None) {
        let mut presets: Vec<String> = selection
            .presets
            .presets
            .iter()
            .map(|preset| preset.name.clone())
            .collect();
        presets.sort();
        if !presets.is_empty() {
            println!("polyx_presets: {}", presets.join(", "));
        }
    }
    if let Ok(selection) = bijux_api::v1::fastq::fastq_banks::resolve_contaminant_selection(None) {
        let mut presets: Vec<String> = selection
            .presets
            .presets
            .iter()
            .map(|preset| preset.name.clone())
            .collect();
        presets.sort();
        if !presets.is_empty() {
            println!("contaminant_presets: {}", presets.join(", "));
        }
    }
}
fn list_fastq_tools(registry: &bijux_api::v1::types::ToolRegistry, stage_id: &str) {
    let mut tools: Vec<_> = registry
        .tools_for_stage(stage_id)
        .into_iter()
        .map(|tool| (tool.tool_id.clone(), tool.role))
        .collect();
    tools.sort_by(|a, b| a.0.cmp(&b.0));
    for (tool_id, role) in tools {
        println!("{tool_id}\t{}", tool_tier_label(role));
    }
}
fn load_adapter_selection(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
) -> Result<AdapterSelection> {
    resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)
}
fn list_adapter_presets(presets: &AdapterPresetsV1) {
    for preset in &presets.presets {
        let categories = if preset.tags.is_empty() {
            "none".to_string()
        } else {
            preset.tags.join(", ")
        };
        println!("{}: categories: {}", preset.name, categories);
    }
}
fn list_adapters(effective: &bijux_api::v1::fastq::EffectiveAdapterSet) {
    println!("preset: {}", effective.preset);
    println!("id\ttags\tname\tread_scope\tenabled_by_default");
    for adapter in &effective.adapters {
        let read_scope = match adapter.read_scope {
            bijux_api::v1::fastq::ReadScope::R1 => "r1",
            bijux_api::v1::fastq::ReadScope::R2 => "r2",
            bijux_api::v1::fastq::ReadScope::Both => "both",
            bijux_api::v1::fastq::ReadScope::SingleEnd => "single_end",
            bijux_api::v1::fastq::ReadScope::PairedEnd => "paired_end",
            bijux_api::v1::fastq::ReadScope::Unknown => "unknown",
        };
        let tags = if adapter.tags.is_empty() {
            "none".to_string()
        } else {
            adapter.tags.join(",")
        };
        println!(
            "{}\t{}\t{}\t{}\t{}",
            adapter.id, tags, adapter.name, read_scope, adapter.enabled_by_default
        );
    }
}
fn tool_tier_label(role: bijux_api::v1::types::ToolRole) -> &'static str {
    match role {
        bijux_api::v1::types::ToolRole::Authoritative => "gold",
        bijux_api::v1::types::ToolRole::Diagnostic => "silver",
        bijux_api::v1::types::ToolRole::Experimental => "experimental",
    }
}
fn set_scientific_preset(preset: Option<cli::parse::ScientificPresetArg>) {
    if let Some(preset) = preset {
        std::env::set_var(
            "BIJUX_SCIENTIFIC_PRESET",
            format!("{preset:?}").to_lowercase(),
        );
    } else {
        std::env::remove_var("BIJUX_SCIENTIFIC_PRESET");
    }
}
fn set_tool_tier_policy(allow_silver: bool, allow_experimental: bool) {
    if allow_silver || allow_experimental {
        std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    } else {
        std::env::remove_var("BIJUX_ALLOW_SILVER");
    }
    if allow_experimental {
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
    } else {
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
    }
}
fn tool_tier_policy_for_fastq(command: &FastqCommand) -> (bool, bool) {
    match command {
        FastqCommand::Trim(args) => (args.common.allow_silver, args.common.allow_experimental),
        FastqCommand::ValidatePre(args) => {
            (args.common.allow_silver, args.common.allow_experimental)
        }
        FastqCommand::Filter(args) => (args.common.allow_silver, args.common.allow_experimental),
        FastqCommand::Preprocess(args) => {
            (args.common.allow_silver, args.common.allow_experimental)
        }
        FastqCommand::Run(args) => (
            args.args.common.allow_silver,
            args.args.common.allow_experimental,
        ),
        FastqCommand::Merge(args)
        | FastqCommand::ErrorCorrect(args)
        | FastqCommand::Qc(args)
        | FastqCommand::Umi(args)
        | FastqCommand::Contam(args)
        | FastqCommand::StatsNeutral(args)
        | FastqCommand::Align(args) => (args.allow_silver, args.allow_experimental),
        _ => (false, false),
    }
}
fn explain_fastq_stage(registry: &bijux_api::v1::types::ToolRegistry, stage_id: &str) -> Result<()> {
    if stage_id == "fastq.preprocess" {
        let args = bijux_api::v1::fastq::fastq_args::BenchFastqPreprocessArgs {
            sample_id: "explain".to_string(),
            profile: None,
            r1: PathBuf::from("reads.fastq.gz"),
            r2: None,
            out: PathBuf::from("artifacts"),
            strict: false,
            auto: false,
            objective: bijux_api::v1::types::Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            adapter_bank_preset: None,
            adapter_bank: Some(format!(
                "preset:{}",
                bijux_api::v1::fastq::fastq_banks::DEFAULT_ADAPTER_PRESET
            )),
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            enable_contaminant_removal: false,
            no_qc_post: false,
            force_merge: false,
            enable_correct: false,
        };
        let plan = bijux_api::v1::plan::fastq_preprocess_plan(&args);
        println!("stage: {stage_id}");
        println!("pipeline:");
        for step in plan.stages {
            println!("- {step}");
        }
        return Ok(());
    }
    let stage = registry
        .stages()
        .get(stage_id)
        .ok_or_else(|| anyhow!("unknown stage {stage_id}"))?;
    println!("stage: {}", stage.stage_id);
    if !stage.description.is_empty() {
        println!("description: {}", stage.description);
    }
    println!("inputs:");
    for input in &stage.inputs {
        println!("- {} ({})", input.name, input.data_type);
    }
    println!("outputs:");
    for output in &stage.outputs {
        println!("- {} ({})", output.name, output.data_type);
    }
    Ok(())
}
