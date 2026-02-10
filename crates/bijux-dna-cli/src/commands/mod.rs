#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::{load_manifests, load_profile, resolve_run_base_dir};
use bijux_dna_api::v1::api::run::{CategorizedError, ErrorCategory};
use bijux_dna_domain_compiler::{domain_coverage_report, validate_domain, ValidateOptions};
use clap::Parser;

struct CwdGuard(PathBuf);

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

pub(crate) mod bam;
pub(crate) mod bench;
pub mod cli;
pub(crate) mod command_prelude;
pub(crate) mod fastq;
pub(crate) mod report_inputs;
pub(crate) mod run_plan;
pub(crate) mod validation;

include!("policies.rs");

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_args(args: &[&str], cwd: &Path) -> Result<()> {
    let argv = std::iter::once("bijux-dna")
        .chain(args.iter().copied())
        .collect::<Vec<_>>();
    let cli = cli::Cli::parse_from(argv);
    run_with_cli(&cli, cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_cli(cli: &cli::Cli, cwd: &Path) -> Result<()> {
    let original_cwd = std::env::current_dir().context("resolve current dir")?;
    std::env::set_current_dir(cwd).context("set current dir")?;
    let _guard = CwdGuard(original_cwd);

    if let Some(path) = &cli.telemetry_jsonl {
        let telemetry_path = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        std::env::set_var("BIJUX_TELEMETRY_JSONL", telemetry_path);
    }
    let domain_dir = cwd.join("domain");
    let registry_path = cwd.join("configs").join("tool_registry.toml");

    if let cli::RootCommand::Environment { command } = &cli.command {
        return handle_environment_root(command, cwd);
    }
    if let cli::RootCommand::Registry { command } = &cli.command {
        return handle_registry_root(command, cwd);
    }
    if let cli::RootCommand::Domain { command } = &cli.command {
        return handle_domain_root(command, cwd);
    }
    if let cli::RootCommand::Lab { command } = &cli.command {
        return handle_lab_root(command, cwd);
    }
    if let cli::RootCommand::Status(args) = &cli.command {
        return handle_status_root(args, cwd);
    }
    let dna_command = match &cli.command {
        cli::RootCommand::Dna { command } => command,
        cli::RootCommand::Environment { .. }
        | cli::RootCommand::Registry { .. }
        | cli::RootCommand::Domain { .. }
        | cli::RootCommand::Lab { .. }
        | cli::RootCommand::Status(_) => {
            unreachable!("handled above")
        }
    };

    if fastq::handle_meta_commands(cli, dna_command, &domain_dir, &registry_path)? {
        return Ok(());
    }

    let profile_path = cwd
        .join("configs")
        .join(format!("profile.{}.toml", cli.profile));
    let mut profile = load_profile(&profile_path).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::PlanError,
            format!("failed to load profile {}: {err}", profile_path.display())
        ))
    })?;
    profile.run_base_dir = resolve_run_base_dir(cwd, &profile.run_base_dir);
    if cli.print_effective_config || cli.dump_effective_config {
        let payload = serde_json::json!({
            "profile": profile,
            "platform": cli.platform,
        });
        cli::render::json::print_pretty(&payload)?;
        return Ok(());
    }

    let registry = load_manifests(&registry_path).map_err(|err| {
        anyhow!(CategorizedError::new(
            ErrorCategory::ContractError,
            format!("manifest validation failed: {err}")
        ))
    })?;

    if bench::handle_fastq_bench(cli, dna_command, &registry)? {
        return Ok(());
    }

    if bam::handle_bam_commands(cli, dna_command, &registry, &domain_dir)? {
        return Ok(());
    }

    run_plan::run_plan(cli, dna_command, &registry, &domain_dir)
}

fn parse_scalar(raw: &str, key: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        let prefix = format!("{key}:");
        if !trimmed.starts_with(&prefix) {
            return None;
        }
        let value = trimmed[prefix.len()..].trim().trim_matches('"');
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn handle_status_root(args: &cli::StatusArgs, cwd: &Path) -> Result<()> {
    let domain_dir = cwd.join("domain");
    let mut planned = Vec::new();
    let mut placeholders = Vec::new();
    let mut missing_fixtures = Vec::new();
    let mut missing_stage_fields = Vec::new();
    let mut missing_tool_fields = Vec::new();
    let normalized_scope = args.scope.replace('-', "_");
    let required_stage_fields = [
        "stage_id",
        "domain",
        "status",
        "scope",
        "inputs",
        "outputs",
        "invariants",
        "compatible_tools",
        "assumptions",
        "metrics_schema",
    ];
    let required_tool_fields = [
        "tool_id",
        "status",
        "scope",
        "default_version",
        "upstream",
        "pin_strategy",
        "license",
        "stage_ids",
        "version_cmd",
        "help_cmd",
        "expected_artifacts",
        "metrics_schema",
    ];

    for dom in ["fastq", "bam"] {
        let stages_dir = domain_dir.join(dom).join("stages");
        if stages_dir.exists() {
            for entry in std::fs::read_dir(&stages_dir)
                .with_context(|| format!("read {}", stages_dir.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                    || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
                {
                    continue;
                }
                let raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                let stage_id = parse_scalar(&raw, "stage_id").unwrap_or_else(|| "<unknown>".into());
                let status = parse_scalar(&raw, "status").unwrap_or_else(|| "supported".into());
                let scope = parse_scalar(&raw, "scope").unwrap_or_else(|| "unknown".into());
                if scope != normalized_scope {
                    continue;
                }
                if status == "planned" || status == "out_of_scope" {
                    planned.push(format!("stage:{stage_id}:{status}"));
                }
                let lower = raw.to_ascii_lowercase();
                if lower.contains("todo") || lower.contains("tbd") || lower.contains("placeholder")
                {
                    placeholders.push(path.display().to_string());
                }
                for key in required_stage_fields {
                    let needle = format!("{key}:");
                    if !raw
                        .lines()
                        .any(|line| line.trim_start().starts_with(&needle))
                    {
                        missing_stage_fields.push(format!(
                            "{} missing required key `{}`",
                            path.display(),
                            key
                        ));
                    }
                }
            }
        }

        let tools_dir = domain_dir.join(dom).join("tools");
        if tools_dir.exists() {
            for entry in std::fs::read_dir(&tools_dir)
                .with_context(|| format!("read {}", tools_dir.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                    || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
                {
                    continue;
                }
                let raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                let tool_id = parse_scalar(&raw, "tool_id").unwrap_or_else(|| "<unknown>".into());
                let status = parse_scalar(&raw, "status").unwrap_or_else(|| "supported".into());
                let scope = parse_scalar(&raw, "scope").unwrap_or_else(|| "unknown".into());
                if scope != normalized_scope {
                    continue;
                }
                if status == "planned" || status == "out_of_scope" {
                    planned.push(format!("tool:{tool_id}:{status}"));
                }
                let lower = raw.to_ascii_lowercase();
                if lower.contains("todo") || lower.contains("tbd") || lower.contains("placeholder")
                {
                    placeholders.push(path.display().to_string());
                }
                for key in required_tool_fields {
                    let needle = format!("{key}:");
                    if !raw
                        .lines()
                        .any(|line| line.trim_start().starts_with(&needle))
                    {
                        missing_tool_fields.push(format!(
                            "{} missing required key `{}`",
                            path.display(),
                            key
                        ));
                    }
                }
            }
        }

        let index = domain_dir.join(dom).join("index.yaml");
        if index.exists() {
            let raw = std::fs::read_to_string(&index)
                .with_context(|| format!("read {}", index.display()))?;
            let mut in_matrix = false;
            for line in raw.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("stage_tool_compatibility:") {
                    in_matrix = true;
                    continue;
                }
                if in_matrix && !line.starts_with("  ") {
                    in_matrix = false;
                }
                if !in_matrix {
                    continue;
                }
                if !(trimmed.contains(':') && trimmed.contains('[') && trimmed.contains(']')) {
                    continue;
                }
                let mut parts = trimmed.splitn(2, ':');
                let Some(stage_id) = parts.next().map(str::trim) else {
                    continue;
                };
                let Some(rhs) = parts.next() else {
                    continue;
                };
                let tools_csv = rhs.trim().trim_start_matches('[').trim_end_matches(']');
                for tool in tools_csv
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                {
                    let fixture = domain_dir
                        .join(dom)
                        .join("fixtures")
                        .join(stage_id)
                        .join(format!("{tool}.txt"));
                    if !fixture.exists() {
                        missing_fixtures.push(fixture.display().to_string());
                    }
                }
            }
        }
    }

    planned.sort();
    planned.dedup();
    placeholders.sort();
    placeholders.dedup();
    missing_fixtures.sort();
    missing_fixtures.dedup();
    missing_stage_fields.sort();
    missing_stage_fields.dedup();
    missing_tool_fields.sort();
    missing_tool_fields.dedup();

    println!("scope={}", args.scope);
    println!("planned_or_out_of_scope={}", planned.len());
    for item in &planned {
        println!("  {item}");
    }
    println!("placeholder_files={}", placeholders.len());
    for item in &placeholders {
        println!("  {item}");
    }
    println!("missing_truth_fixtures={}", missing_fixtures.len());
    for item in &missing_fixtures {
        println!("  {item}");
    }
    println!(
        "missing_stage_required_fields={}",
        missing_stage_fields.len()
    );
    for item in &missing_stage_fields {
        println!("  {item}");
    }
    println!("missing_tool_required_fields={}", missing_tool_fields.len());
    for item in &missing_tool_fields {
        println!("  {item}");
    }

    if let Some(path) = &args.write_checklist {
        let mut md = String::new();
        md.push_str("# Scope Closure Checklist\n\n");
        md.push_str(&format!("- scope: `{}`\n", args.scope));
        md.push_str(&format!("- planned_or_out_of_scope: `{}`\n", planned.len()));
        md.push_str(&format!("- placeholder_files: `{}`\n", placeholders.len()));
        md.push_str(&format!(
            "- missing_truth_fixtures: `{}`\n",
            missing_fixtures.len()
        ));
        md.push_str(&format!(
            "- missing_stage_required_fields: `{}`\n",
            missing_stage_fields.len()
        ));
        md.push_str(&format!(
            "- missing_tool_required_fields: `{}`\n\n",
            missing_tool_fields.len()
        ));

        md.push_str("## Planned / Out Of Scope\n");
        if planned.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &planned {
                md.push_str(&format!("- {item}\n"));
            }
        }
        md.push_str("\n## Placeholder Files\n");
        if placeholders.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &placeholders {
                md.push_str(&format!("- {item}\n"));
            }
        }
        md.push_str("\n## Missing Fixtures\n");
        if missing_fixtures.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_fixtures {
                md.push_str(&format!("- {item}\n"));
            }
        }
        md.push_str("\n## Missing Stage Required Fields\n");
        if missing_stage_fields.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_stage_fields {
                md.push_str(&format!("- {item}\n"));
            }
        }
        md.push_str("\n## Missing Tool Required Fields\n");
        if missing_tool_fields.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_tool_fields {
                md.push_str(&format!("- {item}\n"));
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(path, md).with_context(|| format!("write {}", path.display()))?;
        println!("scope_closure_checklist={}", path.display());
    }
    Ok(())
}

fn handle_environment_root(command: &cli::EnvCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        env_doctor, print_env_export_json, print_env_images, print_env_info,
        print_env_registry_list, run_env_prep, run_env_smoke, run_env_smoke_for_stage,
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
                | cli::EnvCommand::Smoke(_)
                | cli::EnvCommand::Prep(_) => {}
            }
        }
    }
    Ok(())
}

fn handle_registry_root(command: &cli::RegistryCommand, cwd: &Path) -> Result<()> {
    use crate::commands::cli::env::{
        print_registry_coverage_matrix, print_registry_export_json, print_registry_list_stages,
        print_registry_show, print_registry_show_stage, print_registry_show_tool,
        print_registry_tools,
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
