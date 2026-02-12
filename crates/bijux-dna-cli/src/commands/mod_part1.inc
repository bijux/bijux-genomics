
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::{load_manifests, load_profile, resolve_run_base_dir};
use bijux_dna_api::v1::api::run::{CategorizedError, ErrorCategory};
use bijux_dna_domain_compiler::{domain_coverage_report, validate_domain, ValidateOptions};
use clap::Parser;
use sha2::Digest as _;

struct CwdGuard(PathBuf);

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

pub(crate) mod bam;
pub(crate) mod bench;
pub(crate) mod bench_suite;
pub mod cli;
pub(crate) mod command_prelude;
pub(crate) mod corpus;
pub(crate) mod ena;
pub(crate) mod example;
pub(crate) mod fastq;
pub mod hpc;
pub(crate) mod report_inputs;
pub(crate) mod run_plan;
pub(crate) mod validation;
pub(crate) mod vcf;

include!("policies.rs");

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_args(args: &[&str], cwd: &Path) -> Result<()> {
    let normalized_args = if matches!(args.first(), Some(&"bijux" | &"bijux-dna")) {
        &args[1..]
    } else {
        args
    };
    let mut args_with_bin = vec!["bijux"];
    if !matches!(normalized_args.first(), Some(&"dna")) {
        args_with_bin.push("dna");
    }
    args_with_bin.extend(normalized_args.iter().copied());
    let cli = cli::Cli::parse_from(args_with_bin);
    run_with_cli(&cli, cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
#[allow(clippy::too_many_lines)]
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
    if cli.json {
        std::env::set_var("BIJUX_OUTPUT_JSON", "1");
    }
    let domain_dir = cwd.join("domain");
    let registry_path = cwd.join("configs").join("tool_registry.toml");

    let dna_command = match &cli.command {
        cli::RootCommand::Dna { command } => command,
    };
    match dna_command {
        cli::DnaCommand::Environment { command } | cli::DnaCommand::Env { command } => {
            return handle_environment_root(command, cwd);
        }
        cli::DnaCommand::Registry { command } => return handle_registry_root(command, cwd),
        cli::DnaCommand::Ena { command } => return handle_ena_root(command, cwd),
        cli::DnaCommand::Corpus { command } => return handle_corpus_root(command, cwd),
        cli::DnaCommand::Example { command } => return handle_example_root(command, cwd),
        cli::DnaCommand::Plan(args) => return example::print_example_plan(cwd, &args.id),
        cli::DnaCommand::Tool { command } => return handle_tool_root(command, cwd),
        cli::DnaCommand::Domain { command } => return handle_domain_root(command, cwd),
        cli::DnaCommand::Lab { command } => return handle_lab_root(command, cwd),
        cli::DnaCommand::Config { command } => return handle_config_root(command, cwd),
        cli::DnaCommand::Status(args) => return handle_status_root(args, cwd),
        _ => {}
    }

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
    configure_run_context_env(cli, &profile)?;
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

    if let cli::DnaCommand::Dev { command } = dna_command {
        return handle_dev_dna(command, cwd);
    }

    if bench::handle_fastq_bench(cli, dna_command, &registry)? {
        return Ok(());
    }

    if bam::handle_bam_commands(cli, dna_command, &registry, &domain_dir)? {
        return Ok(());
    }
    if vcf::handle_vcf_commands(cli, dna_command)? {
        return Ok(());
    }
    if handle_observability_commands(dna_command, cwd)? {
        return Ok(());
    }

    if matches!(
        dna_command,
        cli::DnaCommand::Fastq { .. } | cli::DnaCommand::Bam { .. } | cli::DnaCommand::Vcf { .. }
    ) {
        let (stage, _, _) = cli::resolve_stage_tool(dna_command);
        enforce_offline_runtime_policy()?;
        ensure_stage_bank_requirements(cwd, stage.as_str())?;
        let run_domain = stage
            .as_str()
            .split('.')
            .next()
            .unwrap_or("fastq")
            .to_string();
        let registry_policy_report =
            crate::commands::cli::env::policy_clean_report(&registry_path, &run_domain)?;
        if !registry_policy_report.ok {
            return Err(anyhow!(
                "run blocked by policy-clean gate for domain `{}`: {}",
                run_domain,
                registry_policy_report
                    .checks
                    .iter()
                    .filter(|check| !check.ok)
                    .map(|check| format!("{}={}", check.name, check.detail))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
        std::env::set_var(
            "BIJUX_POLICY_CLEAN_REPORT_JSON",
            serde_json::to_string(&registry_policy_report)?,
        );
    }

    if cli.profile.eq_ignore_ascii_case("hpc") {
        let (stage, _, _) = cli::resolve_stage_tool(dna_command);
        crate::commands::cli::env::lint_registry_hpc(
            cwd,
            &registry_path,
            stage.as_str().split('.').next(),
            Some(stage.as_str()),
        )
        .map_err(|err| anyhow!("HPC run blocked by registry policy: {err}"))?;
    }

    run_plan::run_plan(cli, dna_command, &registry, &domain_dir)
}

fn configure_run_context_env<T>(cli: &cli::Cli, profile: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let bytes = serde_json::to_vec(profile)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    std::env::set_var("BIJUX_PROFILE_HASH", format!("{:x}", hasher.finalize()));
    if cli.profile.eq_ignore_ascii_case("hpc") {
        std::env::set_var("BIJUX_RUN_CONTEXT", "hpc");
        std::env::set_var("BIJUX_HPC_SITE", "lunarc");
    } else {
        std::env::set_var("BIJUX_RUN_CONTEXT", "local");
    }
    Ok(())
}

fn enforce_offline_runtime_policy() -> Result<()> {
    let allow_network = std::env::var("BIJUX_ALLOW_NETWORK")
        .unwrap_or_else(|_| "0".to_string())
        .to_ascii_lowercase();
    if matches!(allow_network.as_str(), "1" | "true" | "yes") {
        return Err(anyhow!(
            "production run blocked: BIJUX_ALLOW_NETWORK must be disabled (offline policy)"
        ));
    }
    std::env::set_var("BIJUX_ALLOW_NETWORK", "0");
    Ok(())
}

fn ensure_stage_bank_requirements(cwd: &Path, stage_id: &str) -> Result<()> {
    if !stage_requires_banks(stage_id) {
        return Ok(());
    }
    let hpc_root = crate::commands::hpc::load_hpc_config()
        .map_or_else(|_| PathBuf::from("bijux"), |cfg| cfg.resolve_paths().root);
    let candidates = [
        hpc_root.join("bijux-dna-data").join("banks"),
        cwd.join("bijux-dna-data").join("banks"),
        cwd.join("assets"),
    ];
    let mut found = None;
    for path in candidates {
        if path.exists() && path.is_dir() {
            let has_files = std::fs::read_dir(&path)
                .ok()
                .and_then(|mut iter| iter.next().map(|_| true))
                .unwrap_or(false);
            if has_files {
                found = Some(path);
                break;
            }
        }
    }
    if found.is_none() {
        return Err(anyhow!(
            "stage `{stage_id}` requires DB/banks but no non-empty banks directory was found"
        ));
    }
    Ok(())
}

fn stage_requires_banks(stage_id: &str) -> bool {
    let key = stage_id.to_ascii_lowercase();
    key.contains("screen")
        || key.contains("host_depletion")
        || key.contains("prepare_reference")
        || key.contains("detect_adapters")
}

fn handle_observability_commands(dna_command: &cli::DnaCommand, cwd: &Path) -> Result<bool> {
    match dna_command {
        cli::DnaCommand::Debug(args) => {
            if args.view != "tail" {
                return Err(anyhow!(
                    "unsupported --view `{}` (expected `tail`)",
                    args.view
                ));
            }
            let run_dir = cwd.join(&args.search_root).join(&args.run_id);
            let telemetry_path = run_dir.join("run_artifacts").join("telemetry.jsonl");
            let raw = std::fs::read_to_string(&telemetry_path)
                .with_context(|| format!("read {}", telemetry_path.display()))?;
            let mut failure: Option<bijux_dna_runtime::TelemetryEventV1> = None;
            for line in raw.lines() {
                let Ok(event) = serde_json::from_str::<bijux_dna_runtime::TelemetryEventV1>(line)
                else {
                    continue;
                };
                if matches!(
                    event.event_name,
                    bijux_dna_runtime::TelemetryEventName::RunFailed
                ) {
                    failure = Some(event);
                }
            }
            if let Some(event) = failure {
                println!("run_id: {}", event.run_id);
                println!("stage_id: {}", event.stage_id);
                println!("tool_id: {}", event.tool_id);
                println!("status: {}", event.status);
                if let Some(code) = event.failure_code {
                    println!("failure_code: {}", serde_json::to_string(&code)?);
                }
                if let Some(stderr) = event.attrs.get("stderr_path") {
                    println!("stderr_path: {}", serde_json::to_string(stderr)?);
                } else {
                    println!("stderr_path: {}", run_dir.join("logs/stderr.log").display());
                }
            } else {
                println!("no failure events found in {}", telemetry_path.display());
            }
            Ok(true)
        }
        cli::DnaCommand::Collect(args) => {
            let run_dir = cwd.join(&args.search_root).join(&args.run);
            let out = args
                .out
                .clone()
                .unwrap_or_else(|| run_dir.join(format!("{}-log-pack.tar", args.run)));
            let file =
                std::fs::File::create(&out).with_context(|| format!("create {}", out.display()))?;
            let mut archive = tar::Builder::new(file);
            for rel in [
                "run_manifest.json",
                "run_manifest.lock.json",
                "run_artifacts/telemetry.jsonl",
                "logs/stderr.log",
            ] {
                let path = run_dir.join(rel);
                if path.exists() {
                    archive
                        .append_path_with_name(&path, rel)
                        .with_context(|| format!("add {} to log-pack", path.display()))?;
                }
            }
            archive.finish().context("finalize log-pack tar")?;
            println!("{}", out.display());
            Ok(true)
        }
        _ => Ok(false),
    }
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

fn parse_toml_path(path: &Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    raw.parse::<toml::Value>()
        .map_err(|err| anyhow!("parse {}: {err}", path.display()))
}

fn toml_array<'a>(value: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn param_rows(value: &toml::Value) -> Vec<&toml::Value> {
    let rows = toml_array(value, "params");
    if rows.is_empty() {
        toml_array(value, "entries")
    } else {
        rows
    }
}

fn toml_list(value: &toml::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[allow(clippy::too_many_lines)]
fn print_contract_status(cwd: &Path) -> Result<()> {
    let domains = parse_toml_path(&cwd.join("configs/domains.toml"))?;
    let images = parse_toml_path(&cwd.join("configs/images.toml"))?;
    let image_ids = images
        .as_table()
        .map(|table| table.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();

    println!(
        "{:<8} {:<12} {:<7} {:<7} {:<7} {:<7} {:<9} {:<8}",
        "domain", "mode", "stages", "params", "tools", "metrics", "images", "failures"
    );
    println!("{}", "-".repeat(74));

    for domain in toml_array(&domains, "domains") {
        let id = domain
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let experimental = domain
            .get("experimental")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let stages_rel = domain
            .get("stages_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let params_rel = domain
            .get("param_registry_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        let tools_rel = domain
            .get("tool_registry_ssot")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if stages_rel.is_empty() || params_rel.is_empty() || tools_rel.is_empty() {
            println!(
                "{:<8} {:<12} {:<7} {:<7} {:<7} {:<7} {:<9} {:<8}",
                id, "invalid", "-", "-", "-", "-", "-", "yes"
            );
            continue;
        }
        let stages = parse_toml_path(&cwd.join(stages_rel))?;
        let params = parse_toml_path(&cwd.join(params_rel))?;
        let tools = parse_toml_path(&cwd.join(tools_rel))?;

        let param_stage_ids = param_rows(&params)
            .into_iter()
            .filter_map(|row| row.get("stage_id").and_then(toml::Value::as_str))
            .collect::<BTreeSet<_>>();
        let tool_rows = toml_array(&tools, "tools");
        let mut tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
        let mut tool_metrics = BTreeMap::<String, String>::new();
        for row in &tool_rows {
            let tool_id = row
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let metrics_schema = row
                .get("metrics_schema")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .to_string();
            tool_metrics.insert(tool_id.clone(), metrics_schema);
            for stage_id in toml_list(row, "stage_ids") {
                tools_by_stage
                    .entry(stage_id)
                    .or_default()
                    .insert(tool_id.clone());
            }
        }

        let mut stage_count = 0usize;
        let mut missing_params = 0usize;
        let mut missing_tools = 0usize;
        let mut missing_metrics = 0usize;
        let mut missing_images = 0usize;
        for stage in toml_array(&stages, "stages") {
            let stage_id = stage
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or_default();
            if !stage_id.starts_with(&format!("{id}.")) {
                continue;
            }
            let status = stage
                .get("status")
                .and_then(toml::Value::as_str)
                .unwrap_or("supported");
            if status != "supported" {
                continue;
            }
            stage_count += 1;
            if !param_stage_ids.contains(stage_id) {
                missing_params += 1;
            }
            let stage_metrics_schema = stage
                .get("metrics_schema")
                .and_then(toml::Value::as_str)
                .unwrap_or_default();
            let stage_tools = toml_list(stage, "tools")
                .into_iter()
                .chain(
                    tools_by_stage
                        .get(stage_id)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter(),
                )
                .collect::<BTreeSet<_>>();
            let has_metrics = !stage_metrics_schema.trim().is_empty()
                || stage_metrics_schema == "none"
                || stage_tools.iter().any(|tool| {
                    tool_metrics.get(tool).is_some_and(|schema| {
                        !schema.trim().is_empty() && schema != "bijux.unknown.v1"
                    })
                });
            if !has_metrics {
                missing_metrics += 1;
            }
            if stage_tools.is_empty() {
                missing_tools += 1;
            } else if !experimental && stage_tools.iter().any(|tool| !image_ids.contains(tool)) {
                missing_images += 1;
            }
        }
        let failures = missing_params + missing_tools + missing_metrics + missing_images;
        println!(
            "{:<8} {:<12} {:<7} {:<7} {:<7} {:<7} {:<9} {:<8}",
            id,
            if experimental {
                "experimental"
            } else {
                "production"
            },
            stage_count,
            missing_params,
            missing_tools,
            missing_metrics,
            missing_images,
            if failures > 0 { "yes" } else { "no" }
        );
    }
    Ok(())
}

#[allow(clippy::format_push_string, clippy::too_many_lines)]
fn handle_status_root(args: &cli::StatusArgs, cwd: &Path) -> Result<()> {
    if args.scope.eq_ignore_ascii_case("production-readiness") {
        let report =
            crate::commands::bench_suite::production_readiness_status(cwd, "fastq_hpc_01")?;
        cli::render::json::print_pretty(&report)?;
        if report.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(anyhow!("production readiness gate failed"));
        }
        return Ok(());
    }
    if args.hpc {
        let cfg = hpc::load_hpc_config()?;
        let layout = hpc::HpcLayout::from_resolved(&cfg.resolve_paths());
        let report = hpc::validate_hpc_status(&layout);
        cli::render::json::print_pretty(&report)?;
        if !report.ok {
            return Err(anyhow!("hpc status failed"));
        }
        return Ok(());
    }
    if args.contracts {
        return print_contract_status(cwd);
    }
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

    for dom in ["fastq", "bam", "vcf"] {
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
                if lower.contains("todo")
                    || lower.contains("tbd")
                    || lower.contains("placeholder")
                    || lower.contains("sha256:dummy")
                    || lower.contains("0.0.0")
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
                if lower.contains("todo")
                    || lower.contains("tbd")
                    || lower.contains("placeholder")
                    || lower.contains("sha256:dummy")
                    || lower.contains("0.0.0")
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

    if args.placeholders {
        for item in &placeholders {
            println!("{item}");
        }
        return Ok(());
    }

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
            bijux_dna_api::v1::api::run::ensure_dir(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        bijux_dna_api::v1::api::run::atomic_write_bytes(path, md.as_bytes())
            .with_context(|| format!("write {}", path.display()))?;
        println!("scope_closure_checklist={}", path.display());
    }
    Ok(())
}
