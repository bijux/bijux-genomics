#![allow(clippy::too_many_lines)]
//! Root-level command handlers routed from the CLI entrypoint.

use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_compiler::{domain_coverage_report, validate_domain, ValidateOptions};

use crate::commands::{cli, corpus, ena, hpc};

pub(crate) fn handle_ena_root(command: &cli::EnaCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::EnaCommand::Select(args) => ena::select_snapshot(cwd, args)?,
        cli::EnaCommand::Fetch(args) => ena::fetch_from_snapshot(cwd, args)?,
    }
    Ok(())
}

pub(crate) fn handle_corpus_root(command: &cli::CorpusCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::CorpusCommand::Materialize(args) => corpus::materialize_corpus(cwd, args)?,
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

pub(crate) fn handle_registry_root(command: &cli::RegistryCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        lint_registry_hpc, print_registry_audit_fix_suggestions, print_registry_binding_violations,
        print_registry_coverage_matrix, print_registry_doctor,
        print_registry_export_containers_json, print_registry_export_json,
        print_registry_list_stages, print_registry_show, print_registry_show_stage,
        print_registry_show_tool, print_registry_tools, promote_registry_tool,
        verify_registry_tool,
    };
    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
    match command {
        cli::RegistryCommand::Tools { stage, scenario, kind } => {
            print_registry_tools(&registry_path, stage.as_deref(), scenario.as_deref(), kind)?;
        }
        cli::RegistryCommand::Stages => print_registry_list_stages(&registry_path)?,
        cli::RegistryCommand::ShowTool { id } => print_registry_show_tool(&registry_path, id)?,
        cli::RegistryCommand::ShowStage { id } => print_registry_show_stage(&registry_path, id)?,
        cli::RegistryCommand::Show { id } => print_registry_show(&registry_path, id)?,
        cli::RegistryCommand::ExportJson => print_registry_export_json(&registry_path)?,
        cli::RegistryCommand::ExportContainers { json } => {
            if *json {
                print_registry_export_containers_json(&registry_path)?;
            } else {
                return Err(anyhow::anyhow!("registry export-containers requires --json"));
            }
        }
        cli::RegistryCommand::CoverageMatrix => print_registry_coverage_matrix(&registry_path)?,
        cli::RegistryCommand::ValidateTool { id } => verify_registry_tool(&registry_path, id)?,
        cli::RegistryCommand::Audit { show_binding_violations, fix_suggestions, fix_hints } => {
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
        cli::RegistryCommand::Promote { tool } => {
            promote_registry_tool(&registry_path, cwd, tool)?;
        }
        cli::RegistryCommand::Lint { hpc, domain, stages } => {
            if *hpc {
                lint_registry_hpc(cwd, &registry_path, domain.as_deref(), stages.as_deref())?;
            } else {
                print_registry_coverage_matrix(&registry_path)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_tool_root(command: &cli::ToolCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::verify_registry_tool;

    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
    match command {
        cli::ToolCommand::Validate { id } => verify_registry_tool(&registry_path, id)?,
    }
    Ok(())
}

pub(crate) fn handle_environment_root(
    command: &cli::EnvCommand,
    cwd: &Path,
    platform_name: Option<&str>,
) -> Result<()> {
    use crate::commands::cli::env::{
        ensure_apptainer_images, env_doctor, generate_apptainer_qa_matrix_markdown,
        lint_apptainer_defs, parse_stage_domain, print_env_export_json, print_env_images,
        print_env_info, print_env_registry_list, run_env_prep, run_env_smoke,
        run_env_smoke_for_stage, sif_inventory,
    };
    use bijux_dna_api::v1::api::env::{load_image_catalog, load_platform};

    match command {
        cli::EnvCommand::List => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            print_env_registry_list(&registry_path)?;
        }
        cli::EnvCommand::ExportJson => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            print_env_export_json(&registry_path)?;
        }
        cli::EnvCommand::ExportContainers { .. } => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            crate::commands::cli::env::print_registry_export_containers_json(&registry_path)?;
        }
        cli::EnvCommand::ExportHpc { json, hpc_root } => {
            let root = hpc_root
                .clone()
                .map_or_else(|| hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root), Ok)?;
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
            let root = hpc_root
                .clone()
                .map_or_else(|| hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root), Ok)?;
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
            let hpc_root = args
                .hpc_root
                .clone()
                .map_or_else(|| hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root), Ok)?;
            let report = ensure_apptainer_images(
                &bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml"),
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
            let root = hpc_root
                .clone()
                .map_or_else(|| hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root), Ok)?;
            let markdown = generate_apptainer_qa_matrix_markdown(&root)?;
            if let Some(parent) = out.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_api::v1::api::run::atomic_write_bytes(out, markdown.as_bytes())?;
            println!("qa_matrix={}", out.display());
        }
        cli::EnvCommand::EnsureImages(args) => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            let hpc_root = args
                .hpc_root
                .clone()
                .map_or_else(|| hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root), Ok)?;
            let stages = match (&args.stage, &args.stages) {
                (Some(stage), None) => stage.clone(),
                (None, Some(stages)) => stages.clone(),
                _ => {
                    return Err(anyhow::anyhow!(
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
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            if let Some(stage) = args.stage.as_deref() {
                run_env_smoke_for_stage(&registry_path, &args.runtime, stage)?;
            } else if let Some(tool) = args.tool.as_deref() {
                run_env_smoke(&args.runtime, tool)?;
            } else {
                return Err(anyhow::anyhow!("environment smoke requires either <tool> or --stage"));
            }
        }
        cli::EnvCommand::Prep(args) => {
            let registry_path =
                bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
            run_env_prep(
                &registry_path,
                &args.runtime,
                args.tool.as_deref(),
                args.stage.as_deref(),
            )?;
        }
        cli::EnvCommand::Images | cli::EnvCommand::Info | cli::EnvCommand::Doctor => {
            let platform = load_platform(platform_name)
                .map_err(|err| anyhow::anyhow!("failed to load platform: {err}"))?;
            let catalog = load_image_catalog()
                .map_err(|err| anyhow::anyhow!("failed to load images: {err}"))?;
            match command {
                cli::EnvCommand::Images => print_env_images(&catalog, &platform)?,
                cli::EnvCommand::Info => print_env_info(&catalog, &platform),
                cli::EnvCommand::Doctor => env_doctor(&catalog, &platform),
                cli::EnvCommand::List
                | cli::EnvCommand::ExportJson
                | cli::EnvCommand::ExportContainers { .. }
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

pub(crate) fn handle_config_root(command: &cli::ConfigCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::ConfigCommand::InitHpc { root } => {
            let cfg = if let Some(root) = root.clone() {
                hpc::HpcConfig::from_root(root)
            } else {
                hpc::load_hpc_config()
                    .context("config init-hpc requires --root or BIJUX_HPC_CONFIG")?
            };
            let resolved = cfg.resolve_paths();
            let layout = hpc::HpcLayout::from_resolved(&resolved);
            layout.ensure_dirs()?;
            let configs_dir = bijux_dna_infra::configs_dir(cwd);
            bijux_dna_infra::ensure_dir(&configs_dir)?;
            let profiles_dir = configs_dir.join("runtime").join("profiles");
            bijux_dna_infra::ensure_dir(&profiles_dir)?;
            let profile_path = profiles_dir.join("hpc.toml");
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
                return Err(anyhow::anyhow!("config doctor failed"));
            }
        }
        cli::ConfigCommand::CampaignPreflight { config, env_file, user_overrides, json } => {
            let report =
                hpc::campaign_preflight(config, env_file.as_deref(), user_overrides.as_deref())?;
            if *json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("config_path={}", report.config_path);
                println!("env_file_path={}", report.env_file_path);
                println!("user_override_path={}", report.user_override_path);
                println!("user_overrides_applied={}", report.user_overrides_applied);
                println!("ok={}", report.ok);
                println!("slurm_site_profile={}", report.resolved_slurm.site_profile);
                println!("slurm_account={}", report.resolved_slurm.account_redacted);
                println!("slurm_project={}", report.resolved_slurm.project_redacted);
                println!("slurm_partition={}", report.resolved_slurm.partition);
                println!("slurm_qos={}", report.resolved_slurm.qos);
                println!("checks={}", report.checks.len());
            }
            if !report.ok {
                return Err(anyhow::anyhow!("campaign preflight failed"));
            }
        }
        cli::ConfigCommand::CampaignDryRun { config, env_file, user_overrides, json } => {
            let report =
                hpc::campaign_dry_run(config, env_file.as_deref(), user_overrides.as_deref())?;
            if *json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("schema_version={}", report.schema_version);
                println!("config_path={}", report.config_path);
                println!("env_file_path={}", report.env_file_path);
                println!("user_override_path={}", report.user_override_path);
                println!("user_overrides_applied={}", report.user_overrides_applied);
                println!("campaign_id={}", report.campaign_id);
                println!("domain={}", report.domain);
                println!("slurm_site_profile={}", report.resolved_slurm.site_profile);
                println!("slurm_account={}", report.resolved_slurm.account_redacted);
                println!("slurm_project={}", report.resolved_slurm.project_redacted);
                println!("slurm_partition={}", report.resolved_slurm.partition);
                println!("slurm_qos={}", report.resolved_slurm.qos);
                println!("planned_jobs={}", report.planned_jobs.len());
            }
        }
        cli::ConfigCommand::WriteCampaignProfiles { out_dir } => {
            let written = hpc::write_campaign_profiles(out_dir)?;
            for path in written {
                println!("written={}", path.display());
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_slurm_root(command: &cli::SlurmCommand, _cwd: &Path) -> Result<()> {
    match command {
        cli::SlurmCommand::SubmitStageBenchmark(args) => {
            let report = hpc::submit_stage_benchmark(args)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("submission_mode={}", report.mode);
                println!("submitted_jobs={}", report.jobs.len());
            }
        }
        cli::SlurmCommand::SubmitDomainBenchmark(args) => {
            let report = hpc::submit_domain_benchmark(args)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("submission_mode={}", report.mode);
                println!("submitted_jobs={}", report.jobs.len());
            }
        }
        cli::SlurmCommand::SubmitCrossBenchmark(args) => {
            let report = hpc::submit_cross_benchmark(args)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("submission_mode={}", report.mode);
                println!("submitted_jobs={}", report.jobs.len());
            }
        }
        cli::SlurmCommand::SubmitCampaign(args) => {
            let report = hpc::submit_campaign(args)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("submission_mode={}", report.mode);
                println!("submitted_jobs={}", report.jobs.len());
            }
        }
        cli::SlurmCommand::CopyBackManifest(args) => {
            let manifest = hpc::write_copy_back_manifest(args)?;
            if args.json {
                cli::render::json::print_pretty(&manifest)?;
            } else {
                println!("manifest={}", manifest.manifest_path);
                println!("entries={}", manifest.entries.len());
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_domain_root(command: &cli::DomainCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::DomainCommand::Validate { domain_dir } => {
            let path =
                if domain_dir.is_absolute() { domain_dir.clone() } else { cwd.join(domain_dir) };
            validate_domain(&ValidateOptions { domain_dir: path })?;
        }
        cli::DomainCommand::Coverage { domain_dir } => {
            let path =
                if domain_dir.is_absolute() { domain_dir.clone() } else { cwd.join(domain_dir) };
            let payload = domain_coverage_report(&path)?;
            cli::render::json::print_pretty(&payload)?;
        }
    }
    Ok(())
}

pub(crate) fn handle_ci_root(command: &cli::CiCommand, cwd: &Path) -> Result<()> {
    #[derive(serde::Serialize)]
    struct Check {
        name: &'static str,
        ok: bool,
        detail: String,
    }
    #[derive(serde::Serialize)]
    struct Summary {
        schema_version: &'static str,
        ok: bool,
        checks: Vec<Check>,
    }

    let mut checks = Vec::new();

    let workspace_out = cwd.join("artifacts").join("workspace");
    let workspace_ok = crate::commands::workspace_audit(&workspace_out).is_ok();
    checks.push(Check {
        name: "workspace_audit",
        ok: workspace_ok,
        detail: workspace_out.display().to_string(),
    });

    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
    let policy_ok = crate::commands::cli::env::policy_clean_report(&registry_path, "fastq")
        .map(|report| report.ok)
        .unwrap_or(false);
    checks.push(Check {
        name: "registry_policy_clean_fastq",
        ok: policy_ok,
        detail: registry_path.display().to_string(),
    });

    let lint_ok = crate::commands::cli::env::lint_apptainer_defs(cwd).is_ok();
    checks.push(Check {
        name: "lint_apptainer_defs",
        ok: lint_ok,
        detail: "containers/apptainer".to_string(),
    });

    let ok = checks.iter().all(|check| check.ok);
    let summary = Summary { schema_version: "bijux.ci.verify.v1", ok, checks };
    match command {
        cli::CiCommand::Validate { out } => {
            if let Some(parent) = out.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_infra::atomic_write_json(out, &summary)?;
            println!("ci_validate_summary={}", out.display());
            if !ok {
                return Err(anyhow::anyhow!("ci validate failed; see {}", out.display()));
            }
        }
    }
    Ok(())
}

pub(crate) fn handle_lab_root(command: &cli::LabCommand, cwd: &Path) -> Result<()> {
    match command {
        cli::LabCommand::Corpus { command } => match command {
            cli::LabCorpusCommand::ListFastq { corpus, paired } => {
                let root = cwd.join("scripts").join("lab").join("corpus").join("fastq");
                let corpus_root =
                    if corpus == "canonical" { root.join("canonical") } else { root.join(corpus) };
                let scan_root = if corpus_root.exists() { corpus_root } else { root };
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
