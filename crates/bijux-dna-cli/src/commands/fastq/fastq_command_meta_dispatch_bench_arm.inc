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
        }
