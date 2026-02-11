#![allow(clippy::too_many_lines)]

use crate::commands::command_prelude::{
    anyhow, atomic_write_bytes, bench_args_correct, bench_args_filter, bench_args_merge,
    bench_args_preprocess, bench_args_qc_post, bench_args_screen, bench_args_stats,
    bench_args_trim, bench_args_umi, bench_args_validate, bench_fastq_correct, bench_fastq_filter,
    bench_fastq_merge, bench_fastq_preprocess, bench_fastq_qc_post, bench_fastq_screen,
    bench_fastq_stats_neutral, bench_fastq_trim, bench_fastq_umi, bench_fastq_validate_pre, cli,
    compare_runs, compare_runs_with_baseline, env_doctor, load_facts_auto, load_image_catalog,
    load_manifests, load_platform, load_run_summary, objective_spec, print_bench_schema,
    print_env_export_json, print_env_images, print_env_info, print_env_registry_list,
    qc_class_label, render, render_report_bundle_html, resolve_report_inputs, run_env_prep,
    run_env_smoke, run_env_smoke_for_stage, run_image_qa, set_tool_tier_policy, workspace_audit,
    write_correct_report, write_filter_report, write_merge_report, write_qc_post_report,
    write_run_report_from_facts, write_run_summary_from_facts, write_stage_summary_csv,
    write_stats_report, write_trim_report, write_umi_report, write_validate_report, AnalyzeCommand,
    BTreeMap, BenchBamCommand, BenchCommand, BenchFastqCommand, Cli, DnaCommand, EnvCommand,
    Objective, Path, PipelinesCommand, PoliciesCommand, RankInput, Result,
};

pub(crate) fn handle_meta_commands(
    cli: &Cli,
    dna_command: &DnaCommand,
    _domain_dir: &Path,
    registry_path: &Path,
) -> Result<bool> {
    match dna_command {
        DnaCommand::ValidateManifests => {
            let registry = load_manifests(registry_path)
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
        DnaCommand::Platform => {
            let platform = load_platform(cli.platform.as_deref())
                .map_err(|err| anyhow!("failed to load platform: {err}"))?;
            render::json::print_pretty(&platform)?;
            Ok(true)
        }
        DnaCommand::ImageQa => {
            run_image_qa(cli.platform.as_deref())?;
            Ok(true)
        }
        DnaCommand::Replay(args) => {
            if let Some(manifest_path) = args.manifest.as_ref() {
                bijux_dna_api::v1::api::run::replay_manifest(manifest_path, args.verify_only)?;
                return Ok(true);
            }
            let manifest_path = args
                .search_root
                .join(&args.run_id)
                .join("run_manifest.json");
            bijux_dna_api::v1::api::run::replay_manifest(&manifest_path, args.verify_only)?;
            Ok(true)
        }
        DnaCommand::Compare(args) => {
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
            bijux_dna_api::v1::api::run::ensure_dir(output_dir)?;
            let path = output_dir.join("compare.json");
            atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                .map_err(anyhow::Error::from)?;
            render::json::print_pretty(&result)?;
            Ok(true)
        }
        DnaCommand::Policies { command } => {
            match command {
                PoliciesCommand::Audit { out } => {
                    workspace_audit(out)?;
                }
            }
            Ok(true)
        }
        DnaCommand::Pipelines { command } => match command {
            PipelinesCommand::List {
                domain,
                show_experimental,
            } => {
                let profiles = bijux_dna_api::v1::api::plan::select_pipelines(
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
            PipelinesCommand::Explain { id, explain_io } => {
                let profile = bijux_dna_api::v1::api::plan::select_pipelines(None, true)
                    .into_iter()
                    .find(|profile| profile.id.as_str() == id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {id}"))?;
                let io_graph = if *explain_io {
                    let registry = load_manifests(registry_path)?;
                    let nodes = profile
                        .capabilities
                        .required_stages
                        .iter()
                        .filter_map(|stage_id| {
                            let key = bijux_dna_api::v1::api::run::StageId::new(
                                (*stage_id).to_string(),
                            );
                            registry.stages().get(&key).map(|stage| {
                                serde_json::json!({
                                    "stage_id": stage_id,
                                    "semantic_kind": stage.semantic_kind,
                                    "input_kind": stage.input_kind,
                                    "output_kind": stage.output_kind,
                                    "produced_artifacts": stage.produced_artifacts,
                                    "consumes": stage.inputs.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                                    "produces": stage.outputs.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                                })
                            })
                        })
                        .collect::<Vec<_>>();
                    serde_json::Value::Array(nodes)
                } else {
                    serde_json::Value::Null
                };
                let payload = serde_json::json!({
                    "profile": profile,
                    "defaults_ledger": profile.defaults_ledger(),
                    "promised_outputs": profile.capabilities.produces_outputs,
                    "report_sections": profile.capabilities.report_sections,
                    "artifact_graph": io_graph,
                });
                render::json::print_pretty(&payload)?;
                Ok(true)
            }
            PipelinesCommand::ExplainProfile { id } => {
                let resolved_id = resolve_profile_alias(id);
                let profile = bijux_dna_api::v1::api::plan::select_pipelines(None, true)
                    .into_iter()
                    .find(|profile| profile.id.as_str() == resolved_id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {id}"))?;
                let has_fastq = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("fastq."));
                let has_bam = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("bam."));
                let has_vcf = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("vcf."));
                let invariants = match (has_fastq, has_bam, has_vcf) {
                    (true, false, false) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_fastq_profile(&profile),
                    )?,
                    (false, true, false) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_bam_profile(&profile),
                    )?,
                    (false, false, true) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_vcf_profile(&profile),
                    )?,
                    _ => serde_json::Value::Null,
                };
                let payload = serde_json::json!({
                    "profile_id_input": id,
                    "profile_id_resolved": resolved_id,
                    "library_model": profile.library_model,
                    "effective_params": profile.defaults.params,
                    "effective_tools": profile.defaults.tools,
                    "default_rationale": profile.defaults.rationales,
                    "rationale_links": [
                        "docs/SCIENTIFIC_DEFAULTS.md",
                        "docs/20-science/SCIENTIFIC_DECISIONS.md",
                        "crates/bijux-dna-pipelines/docs/PROFILE_RATIONALE.md"
                    ],
                    "invariants": invariants,
                });
                render::json::print_pretty(&payload)?;
                Ok(true)
            }
            PipelinesCommand::ValidateProfile { id } => {
                let resolved_id = resolve_profile_alias(id);
                let profile = bijux_dna_api::v1::api::plan::select_pipelines(None, true)
                    .into_iter()
                    .find(|profile| profile.id.as_str() == resolved_id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {id}"))?;
                let has_fastq = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("fastq."));
                let has_bam = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("bam."));
                let has_vcf = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("vcf."));
                let payload = match (has_fastq, has_bam, has_vcf) {
                    (true, false, false) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_fastq_profile(&profile),
                    )?,
                    (false, true, false) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_bam_profile(&profile),
                    )?,
                    (false, false, true) => serde_json::to_value(
                        bijux_dna_api::v1::api::plan::validate_vcf_profile(&profile),
                    )?,
                    _ => serde_json::json!({
                        "profile_id": resolved_id,
                        "valid": false,
                        "violations": [{
                            "code": "unsupported_domain_mix",
                            "message": "profile must map to exactly one of fastq or bam domains for validate-profile"
                        }]
                    }),
                };
                render::json::print_pretty(&payload)?;
                Ok(true)
            }
            PipelinesCommand::ProfileDiff { left, right } => {
                let left_id = resolve_profile_alias(left);
                let right_id = resolve_profile_alias(right);
                let profiles = bijux_dna_api::v1::api::plan::select_pipelines(None, true);
                let left_profile = profiles
                    .iter()
                    .find(|profile| profile.id.as_str() == left_id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {left}"))?;
                let right_profile = profiles
                    .iter()
                    .find(|profile| profile.id.as_str() == right_id)
                    .ok_or_else(|| anyhow!("unknown pipeline profile: {right}"))?;
                let left_has_fastq = left_profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("fastq."));
                let left_has_vcf = left_profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("vcf."));
                let right_has_fastq = right_profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("fastq."));
                let right_has_vcf = right_profile
                    .capabilities
                    .required_stages
                    .iter()
                    .any(|stage| stage.starts_with("vcf."));
                let payload = serde_json::json!({
                    "left": left_profile.id,
                    "right": right_profile.id,
                    "tools_left": left_profile.defaults.tools,
                    "tools_right": right_profile.defaults.tools,
                    "params_left": left_profile.defaults.params,
                    "params_right": right_profile.defaults.params,
                    "invariants_left": if left_has_fastq {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_fastq_profile(left_profile))?
                    } else if left_has_vcf {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_vcf_profile(left_profile))?
                    } else {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_bam_profile(left_profile))?
                    },
                    "invariants_right": if right_has_fastq {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_fastq_profile(right_profile))?
                    } else if right_has_vcf {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_vcf_profile(right_profile))?
                    } else {
                        serde_json::to_value(bijux_dna_api::v1::api::plan::validate_bam_profile(right_profile))?
                    },
                });
                render::json::print_pretty(&payload)?;
                Ok(true)
            }
            PipelinesCommand::Audit {
                domain,
                show_experimental,
            } => {
                let profiles = bijux_dna_api::v1::api::plan::select_pipelines(
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
                    let id_catalog = match profile.id.as_str() {
                        "fastq-to-fastq__default__v1" | "fastq-to-fastq__minimal__v1" => {
                            bijux_dna_api::v1::api::plan::fastq_pipeline_id_catalog(
                                profile.id.as_str(),
                            )
                        }
                        "fastq-to-bam__default__v1" | "fastq-to-bam__adna_shotgun__v1" => {
                            bijux_dna_api::v1::api::plan::cross_fastq_to_bam_id_catalog(
                                profile.id.as_str(),
                            )
                        }
                        "bam-to-bam__default__v1"
                        | "bam-to-bam__adna_shotgun__v1"
                        | "bam-to-bam__adna_capture__v1" => {
                            bijux_dna_api::v1::api::plan::bam_pipeline_id_catalog(
                                profile.id.as_str(),
                            )
                        }
                        _ => Vec::new(),
                    };
                    for stage_id in id_catalog {
                        if stage_id.starts_with("bam.") {
                            let stage = bijux_dna_api::v1::api::bench::BamStage::try_from(
                                stage_id.as_str(),
                            )
                            .map_err(|_| anyhow!("unknown BAM stage {stage_id}"))?;
                            let completeness =
                                bijux_dna_api::v1::api::bench::bam_stage_completeness(stage);
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
        DnaCommand::Analyze { command } => {
            match command {
                AnalyzeCommand::Runs(args) => {
                    let query = bijux_dna_api::v1::api::run::RunQuery {
                        stage: args.stage.clone(),
                        tool: args.tool.clone(),
                        objective: args.objective.map(|obj| obj.as_str().to_string()),
                        success: args.success,
                    };
                    let runs = bijux_dna_api::v1::api::run::query_runs(&args.index, &query)?;
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
                    bijux_dna_api::v1::api::run::ensure_dir(output_dir)?;
                    let path = output_dir.join("compare.json");
                    atomic_write_bytes(&path, &serde_json::to_vec_pretty(&result)?)
                        .map_err(anyhow::Error::from)?;
                    render::json::print_pretty(&result)?;
                }
                AnalyzeCommand::Rank(args) => {
                    let run_dir = args.search_root.join(&args.run_id);
                    let facts_path = run_dir.join("facts.jsonl");
                    let facts = load_facts_auto(&facts_path)?;
                    let mut by_tool: BTreeMap<
                        String,
                        Vec<&bijux_dna_api::v1::api::run::FactsRowV1>,
                    > = BTreeMap::new();
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
                    let rankings = bijux_dna_api::v1::api::bench::build_rankings(&inputs)?;
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
                                bijux_dna_api::v1::api::run::ensure_dir(&bundle_dir)?;
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
                AnalyzeCommand::Metrics(args) => {
                    let run_dir = args.search_root.join(&args.run_id);
                    let facts_path = run_dir.join("facts.jsonl");
                    let facts = load_facts_auto(&facts_path)?;
                    let mut stage_metrics: BTreeMap<String, serde_json::Value> = BTreeMap::new();
                    for row in facts {
                        if row.stage_id.starts_with("fastq.") {
                            stage_metrics.insert(row.stage_id.clone(), row.metrics.clone());
                        }
                    }
                    let summary = serde_json::json!({
                        "schema_version": "bijux.metrics.summary.v1",
                        "run_id": args.run_id,
                        "stages": stage_metrics,
                    });
                    render::json::print_pretty(&summary)?;
                }
            }
            Ok(true)
        }
        DnaCommand::Env { command } => {
            match command {
                EnvCommand::List => {
                    let cwd = std::env::current_dir()?;
                    let registry_path = cwd.join("configs").join("tool_registry.toml");
                    print_env_registry_list(&registry_path)?;
                }
                EnvCommand::ExportJson => {
                    let cwd = std::env::current_dir()?;
                    let registry_path = cwd.join("configs").join("tool_registry.toml");
                    print_env_export_json(&registry_path)?;
                }
                EnvCommand::ExportHpc { json } => {
                    let root = std::env::var("BIJUX_HPC_ROOT").map_or_else(
                        |_| std::path::PathBuf::from("/home/bijan/bijux"),
                        std::path::PathBuf::from,
                    );
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
                    let registry_path = cwd.join("configs").join("tool_registry.toml");
                    let report = crate::commands::cli::env::ensure_apptainer_images(
                        &registry_path,
                        &args.domain,
                        &args.stages,
                        args.force_smoke,
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
                EnvCommand::Smoke(args) => {
                    let cwd = std::env::current_dir()?;
                    let registry_path = cwd.join("configs").join("tool_registry.toml");
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
                    let registry_path = cwd.join("configs").join("tool_registry.toml");
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
        DnaCommand::Fastq { .. }
        | DnaCommand::Bam { .. }
        | DnaCommand::Vcf { .. }
        | DnaCommand::Debug(_)
        | DnaCommand::Collect(_) => Ok(false),
    }
}

fn resolve_profile_alias(id: &str) -> &str {
    match id {
        "fastq-adna" => "fastq-to-fastq__adna__v1",
        "fastq-reference-adna" => "fastq-to-fastq__reference_adna__v1",
        "fastq-default" => "fastq-to-fastq__default__v1",
        "bam-adna" => "bam-to-bam__adna_shotgun__v1",
        "bam-default" => "bam-to-bam__default__v1",
        "vcf-minimal" => "vcf-to-vcf__minimal__v1",
        other => other,
    }
}

fn bench_bam_stage_args_to_api(
    args: &crate::commands::cli::parse::BenchBamStageArgs,
) -> bijux_dna_api::v1::api::bench::BenchBamStageArgs {
    bijux_dna_api::v1::api::bench::BenchBamStageArgs {
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
        allow_planned: args.allow_planned,
    }
}

fn bench_bam_pipeline_args_to_api(
    args: &crate::commands::cli::parse::BenchBamPipelineArgs,
) -> bijux_dna_api::v1::api::bench::BenchBamPipelineArgs {
    bijux_dna_api::v1::api::bench::BenchBamPipelineArgs {
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
        allow_planned: args.allow_planned,
    }
}
