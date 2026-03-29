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
        cli::RegistryCommand::Tools {
            stage,
            scenario,
            kind,
        } => {
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
        cli::RegistryCommand::Promote { tool } => {
            promote_registry_tool(&registry_path, cwd, tool)?;
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

pub(crate) fn handle_tool_root(command: &cli::ToolCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::verify_registry_tool;

    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");
    match command {
        cli::ToolCommand::Validate { id } => verify_registry_tool(&registry_path, id)?,
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
    }
    Ok(())
}

pub(crate) fn handle_domain_root(command: &cli::DomainCommand, cwd: &Path) -> Result<()> {
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
