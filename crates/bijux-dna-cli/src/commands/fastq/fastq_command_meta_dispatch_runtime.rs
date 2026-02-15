        DnaCommand::Environment { command } => {
            match command {
                EnvCommand::List => {
                    let cwd = std::env::current_dir()?;
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    print_env_registry_list(&registry_path)?;
                }
                EnvCommand::ExportJson => {
                    let cwd = std::env::current_dir()?;
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
                    print_env_export_json(&registry_path)?;
                }
                EnvCommand::ExportContainers { json } => {
                    if !json {
                        return Err(anyhow!("environment export-containers requires --json"));
                    }
                    let cwd = std::env::current_dir()?;
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
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
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
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
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
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
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
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
                    let registry_path = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
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
        DnaCommand::Bench { command } => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
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
                    BenchFastqCommand::Validate(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_validate(args)?;
                        let outcome =
                            bench_fastq_validate_pre(&catalog, &platform, None, &bench_args)?;
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
                    BenchFastqCommand::QcPost(args) => {
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
                    BenchFastqCommand::Screen(args) => {
                        set_tool_tier_policy(false, args.allow_experimental);
                        let bench_args = bench_args_screen(args)?;
                        bench_fastq_screen(&catalog, &platform, None, &bench_args)?;
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
        },
        _ => Ok(false),
