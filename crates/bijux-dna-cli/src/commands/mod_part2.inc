#[allow(clippy::too_many_lines)]
fn handle_environment_root(command: &cli::EnvCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        ensure_apptainer_images, env_doctor, generate_apptainer_qa_matrix_markdown,
        lint_apptainer_defs, parse_stage_domain, print_env_export_json, print_env_images,
        print_env_info, print_env_registry_list, run_env_prep, run_env_smoke,
        run_env_smoke_for_stage, sif_inventory,
    };
    use bijux_dna_api::v1::api::env::{load_image_catalog, load_platform};
    match command {
        cli::EnvCommand::List => {
            let registry_path = cwd.join("configs").join("tool_registry.toml");
            print_env_registry_list(&registry_path)?;
        }
        cli::EnvCommand::ExportJson => {
            let registry_path = cwd.join("configs").join("tool_registry.toml");
            print_env_export_json(&registry_path)?;
        }
        cli::EnvCommand::ExportHpc { json, hpc_root } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let layout = hpc::HpcLayout::from_root(&root);
            let export = hpc::export_hpc_env_json(&layout)?;
            if *json {
                cli::render::json::print_pretty(&export)?;
            } else {
                println!("containers_dir={}", export.containers_dir);
                println!("sif_count={}", export.sifs.len());
            }
        }
        cli::EnvCommand::SifInventory { hpc_root, json } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = sif_inventory(&root)?;
            if *json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("containers_dir={}", report.containers_dir);
                println!("sif_count={}", report.entries.len());
            }
        }
        cli::EnvCommand::Ensure(args) => {
            let domain = parse_stage_domain(&args.stage)?;
            let hpc_root = args.hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let report = ensure_apptainer_images(
                &cwd.join("configs").join("tool_registry.toml"),
                &hpc_root,
                &domain,
                &args.stage,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        cli::EnvCommand::ApptainerQaMatrix { hpc_root, out } => {
            let root = hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
                Ok,
            )?;
            let markdown = generate_apptainer_qa_matrix_markdown(&root)?;
            if let Some(parent) = out.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_api::v1::api::run::atomic_write_bytes(out, markdown.as_bytes())?;
            println!("qa_matrix={}", out.display());
        }
        cli::EnvCommand::EnsureImages(args) => {
            let registry_path = cwd.join("configs").join("tool_registry.toml");
            let hpc_root = args.hpc_root.clone().map_or_else(
                || hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root),
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
            let report = ensure_apptainer_images(
                &registry_path,
                &hpc_root,
                &args.domain,
                &stages,
                args.force_smoke,
                args.repair_mismatch,
            )?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("requested_tools={}", report.tools.len());
                println!("built={}", report.built);
                println!("reused={}", report.reused);
                println!("quick_smoked={}", report.quick_smoked);
                println!("failed={}", report.failed);
            }
        }
        cli::EnvCommand::LintApptainerDefs => {
            lint_apptainer_defs(cwd)?;
        }
        cli::EnvCommand::Smoke(args) => {
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
        cli::EnvCommand::Prep(args) => {
            let registry_path = cwd.join("configs").join("tool_registry.toml");
            run_env_prep(
                &registry_path,
                &args.runtime,
                args.tool.as_deref(),
                args.stage.as_deref(),
            )?;
        }
        cli::EnvCommand::Images | cli::EnvCommand::Info | cli::EnvCommand::Doctor => {
            let platform =
                load_platform(None).map_err(|err| anyhow!("failed to load platform: {err}"))?;
            let catalog =
                load_image_catalog().map_err(|err| anyhow!("failed to load images: {err}"))?;
            match command {
                cli::EnvCommand::Images => print_env_images(&catalog, &platform)?,
                cli::EnvCommand::Info => print_env_info(&catalog, &platform),
                cli::EnvCommand::Doctor => env_doctor(&catalog, &platform),
                cli::EnvCommand::List
                | cli::EnvCommand::ExportJson
                | cli::EnvCommand::ExportHpc { .. }
                | cli::EnvCommand::SifInventory { .. }
                | cli::EnvCommand::Ensure(_)
                | cli::EnvCommand::ApptainerQaMatrix { .. }
                | cli::EnvCommand::EnsureImages(_)
                | cli::EnvCommand::LintApptainerDefs
                | cli::EnvCommand::Smoke(_)
                | cli::EnvCommand::Prep(_) => {}
            }
        }
    }
    Ok(())
}

fn handle_config_root(command: &cli::ConfigCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::ConfigCommand::InitHpc { root } => {
            let cfg = root.clone().map_or_else(
                || {
                    hpc::load_hpc_config().unwrap_or_else(|_| {
                        hpc::HpcConfig::from_root(std::env::var_os("HOME").map_or_else(
                            || PathBuf::from("bijux"),
                            |h| PathBuf::from(h).join("bijux"),
                        ))
                    })
                },
                hpc::HpcConfig::from_root,
            );
            let resolved = cfg.resolve_paths();
            let layout = hpc::HpcLayout::from_resolved(&resolved);
            layout.ensure_dirs()?;
            let configs_dir = cwd.join("configs");
            bijux_dna_infra::ensure_dir(&configs_dir)?;
            let profile_path = configs_dir.join("profile.hpc.toml");
            bijux_dna_api::v1::api::run::atomic_write_bytes(
                &profile_path,
                layout.profile_hpc_toml().as_bytes(),
            )?;
            let config_path = hpc::write_hpc_config(&cfg)?;
            let lock_path = hpc::write_site_lock(&layout)?;
            println!("written={}", profile_path.display());
            println!("hpc_config={}", config_path.display());
            println!("site_lock={}", lock_path.display());
        }
        cli::ConfigCommand::Doctor => {
            let report = hpc::config_doctor()?;
            cli::render::json::print_pretty(&report)?;
            if !report.ok {
                return Err(anyhow!("config doctor failed"));
            }
        }
    }
    Ok(())
}

fn handle_registry_root(command: &cli::RegistryCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        lint_registry_hpc, print_registry_audit_fix_suggestions, print_registry_binding_violations,
        print_registry_coverage_matrix, print_registry_doctor, print_registry_export_json,
        print_registry_list_stages, print_registry_show, print_registry_show_stage,
        print_registry_show_tool, print_registry_tools, promote_registry_tool,
        verify_registry_tool,
    };
    let registry_path = cwd.join("configs").join("tool_registry.toml");
    match command {
        cli::RegistryCommand::Tools { stage, kind } => {
            print_registry_tools(&registry_path, stage.as_deref(), kind)?;
        }
        cli::RegistryCommand::Stages => print_registry_list_stages(&registry_path)?,
        cli::RegistryCommand::ShowTool { id } => print_registry_show_tool(&registry_path, id)?,
        cli::RegistryCommand::ShowStage { id } => print_registry_show_stage(&registry_path, id)?,
        cli::RegistryCommand::Show { id } => print_registry_show(&registry_path, id)?,
        cli::RegistryCommand::ExportJson => print_registry_export_json(&registry_path)?,
        cli::RegistryCommand::CoverageMatrix => print_registry_coverage_matrix(&registry_path)?,
        cli::RegistryCommand::VerifyTool { id } => verify_registry_tool(&registry_path, id)?,
        cli::RegistryCommand::Audit {
            show_binding_violations,
            fix_suggestions,
            fix_hints,
        } => {
            if *show_binding_violations {
                print_registry_binding_violations(&registry_path, None)?;
            } else if *fix_suggestions || *fix_hints {
                print_registry_audit_fix_suggestions(&registry_path)?;
            } else {
                print_registry_export_json(&registry_path)?;
            }
        }
        cli::RegistryCommand::Doctor { domain } => {
            print_registry_doctor(&registry_path, domain.as_deref())?;
        }
        cli::RegistryCommand::Promote { id } => {
            promote_registry_tool(&registry_path, cwd, id)?;
        }
        cli::RegistryCommand::Lint {
            hpc,
            domain,
            stages,
        } => {
            if *hpc {
                lint_registry_hpc(cwd, &registry_path, domain.as_deref(), stages.as_deref())?;
            } else {
                print_registry_coverage_matrix(&registry_path)?;
            }
        }
    }
    Ok(())
}

fn handle_ena_root(command: &cli::EnaCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::EnaCommand::Select(args) => ena::select_snapshot(cwd, args)?,
        cli::EnaCommand::Fetch(args) => ena::fetch_from_snapshot(cwd, args)?,
    }
    Ok(())
}

fn handle_corpus_root(command: &cli::CorpusCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::CorpusCommand::Normalize { corpus } => corpus::normalize_corpus(cwd, corpus)?,
        cli::CorpusCommand::Validate { corpus } => corpus::validate_corpus(cwd, corpus)?,
        cli::CorpusCommand::List(args) => {
            if args.json {
                corpus::list_corpus_json(cwd, args.corpus.as_deref())?;
            } else {
                corpus::list_corpus_text(cwd, args.corpus.as_deref())?;
            }
        }
        cli::CorpusCommand::Diff { left, right, json } => {
            if *json {
                corpus::diff_manifests_json(cwd, left, right)?;
            } else {
                corpus::diff_manifests_text(cwd, left, right)?;
            }
        }
    }
    Ok(())
}

fn handle_example_root(command: &cli::ExampleCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::ExampleCommand::Run(args) => example::run_example(cwd, &args.id, args.hpc)?,
        cli::ExampleCommand::Validate(args) => example::validate_example(cwd, &args.id)?,
        cli::ExampleCommand::Plan(args) => example::print_example_plan(cwd, &args.id)?,
        cli::ExampleCommand::List(args) => {
            if args.json {
                example::list_examples_json(cwd)?;
            } else {
                example::list_examples_text(cwd)?;
            }
        }
        cli::ExampleCommand::Chain(args) => {
            example::chain_examples(cwd, &args.from, &args.arrow, &args.to)?;
        }
    }
    Ok(())
}

fn handle_dev_dna(command: &cli::DevCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::DevCommand::Examples { command } => match command {
            cli::DevExamplesCommand::Scaffold(args) => {
                example::scaffold_examples_series(cwd, &args.series, args.count)?;
            }
        },
    }
    Ok(())
}

fn handle_tool_root(command: &cli::ToolCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::verify_registry_tool;
    let registry_path = cwd.join("configs").join("tool_registry.toml");
    match command {
        cli::ToolCommand::Verify { id } => verify_registry_tool(&registry_path, id)?,
    }
    Ok(())
}

fn handle_lab_root(command: &cli::LabCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::LabCommand::Corpus { command } => match command {
            cli::LabCorpusCommand::ListFastq { corpus, paired } => {
                let root = cwd.join("scripts").join("lab").join("corpus").join("fastq");
                let corpus_root = if corpus == "canonical" {
                    root.join("canonical")
                } else {
                    root.join(corpus)
                };
                let scan_root = if corpus_root.exists() {
                    corpus_root
                } else {
                    root
                };
                let mut stack = vec![scan_root];
                let mut files = Vec::new();
                while let Some(dir) = stack.pop() {
                    for entry in std::fs::read_dir(&dir)
                        .with_context(|| format!("read {}", dir.display()))?
                    {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                            continue;
                        }
                        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                            continue;
                        };
                        let is_fastq = name.ends_with(".fastq.gz");
                        if !is_fastq {
                            continue;
                        }
                        if *paired {
                            if name.ends_with("_R1.fastq.gz") || name.ends_with("_1.fastq.gz") {
                                files.push(path);
                            }
                        } else {
                            files.push(path);
                        }
                    }
                }
                files.sort();
                files.dedup();
                for file in files {
                    println!("{}", file.display());
                }
            }
        },
    }
    Ok(())
}

fn handle_domain_root(command: &cli::DomainCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::DomainCommand::Validate { domain_dir } => {
            let path = if domain_dir.is_absolute() {
                domain_dir.clone()
            } else {
                cwd.join(domain_dir)
            };
            validate_domain(&ValidateOptions { domain_dir: path })?;
        }
        cli::DomainCommand::Coverage { domain_dir } => {
            let path = if domain_dir.is_absolute() {
                domain_dir.clone()
            } else {
                cwd.join(domain_dir)
            };
            let payload = domain_coverage_report(&path)?;
            cli::render::json::print_pretty(&payload)?;
        }
    }
    Ok(())
}
