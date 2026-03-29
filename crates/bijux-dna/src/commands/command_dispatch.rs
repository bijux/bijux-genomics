use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::{load_manifests, load_profile, resolve_run_base_dir};
use bijux_dna_api::v1::api::run::{CategorizedError, ErrorCategory};

use crate::commands::root::{
    handle_ci_root, handle_config_root, handle_corpus_root, handle_domain_root,
    handle_ena_root, handle_environment_root, handle_lab_root, handle_registry_root,
    handle_tool_root,
};
pub use crate::commands::router::argv::{parse_cli_from_argv, parse_process_cli};
use crate::commands::router::runtime::{
    configure_process_cli_env, configure_run_context_env, enter_cli_cwd,
};
use crate::commands::{bam, bench, cli, fastq, hpc, run_plan, vcf};

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_args(args: &[&str], cwd: &Path) -> Result<()> {
    let argv_parts = args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let cli = parse_cli_from_argv(&argv_parts)?;
    run_with_cli(&cli, cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_cli(cli: &cli::Cli, cwd: &Path) -> Result<()> {
    let _guard = enter_cli_cwd(cwd)?;
    configure_process_cli_env(cli, cwd);
    let domain_dir = cwd.join("domain");
    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");

    let dna_command = &cli.command;
    match dna_command {
        cli::DnaCommand::Environment(args) => {
            return handle_environment_root(&args.command, cwd, cli.platform.as_deref());
        }
        cli::DnaCommand::Registry(args) => return handle_registry_root(&args.command, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Ena(args) => return handle_ena_root(&args.command, cwd),
        cli::DnaCommand::Corpus(args) => return handle_corpus_root(&args.command, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Tool(args) => return handle_tool_root(&args.command, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Domain(args) => return handle_domain_root(&args.command, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Lab(args) => return handle_lab_root(&args.command, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Config(args) => return handle_config_root(&args.command, cwd),
        cli::DnaCommand::Status(args) => return handle_status_root(args, cwd),
        #[cfg(debug_assertions)]
        cli::DnaCommand::Ci(args) => return handle_ci_root(&args.command, cwd),
        _ => {}
    }

    if fastq::handle_meta_commands(cli, dna_command, &domain_dir, &registry_path)? {
        return Ok(());
    }

    let profile_path =
        bijux_dna_infra::configs_file(cwd, &format!("runtime/profiles/{}.toml", cli.profile));
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

    if matches!(dna_command, cli::DnaCommand::Fastq(..)) || {
        #[cfg(debug_assertions)]
        {
            matches!(
                dna_command,
                cli::DnaCommand::Bam(..) | cli::DnaCommand::Vcf(..)
            )
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    } {
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
    let mut candidates = Vec::new();
    if let Ok(hpc_root) = crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root)
    {
        candidates.push(hpc_root.join("bijux-dna-data").join("banks"));
    }
    candidates.push(cwd.join("examples").join("bijux-dna-data").join("banks"));
    candidates.push(cwd.join("assets"));
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
    #[cfg(not(debug_assertions))]
    let _ = cwd;

    match dna_command {
        #[cfg(debug_assertions)]
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
        #[cfg(debug_assertions)]
        cli::DnaCommand::Collect(args) => {
            let run_dir = cwd.join(&args.search_root).join(&args.run);
            let out = args
                .out
                .clone()
                .unwrap_or_else(|| run_dir.join(format!("{}-log-pack.tar", args.run)));
            let file = bijux_dna_infra::create_file(&out)
                .with_context(|| format!("create {}", out.display()))?;
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

fn toml_array<'a>(value: &'a toml::Value, key: &str) -> Result<Vec<&'a toml::Value>> {
    let Some(raw) = value.get(key) else {
        return Ok(Vec::new());
    };
    let rows = raw
        .as_array()
        .ok_or_else(|| anyhow!("registry field `{key}` must be an array"))?;
    Ok(rows.iter().collect::<Vec<_>>())
}

fn param_rows(value: &toml::Value) -> Result<Vec<&toml::Value>> {
    let rows = toml_array(value, "params")?;
    if rows.is_empty() {
        toml_array(value, "entries")
    } else {
        Ok(rows)
    }
}

fn toml_list(value: &toml::Value, key: &str) -> Result<Vec<String>> {
    let Some(raw) = value.get(key) else {
        return Ok(Vec::new());
    };
    let rows = raw
        .as_array()
        .ok_or_else(|| anyhow!("registry field `{key}` must be an array"))?;
    rows.iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| anyhow!("registry field `{key}` must contain non-empty strings"))
        })
        .collect()
}

fn declared_toml_str<'a>(value: &'a toml::Value, key: &str) -> Option<&'a str> {
    value.get(key)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn declared_toml_array<'a>(value: &'a toml::Value, key: &str) -> Option<Vec<&'a toml::Value>> {
    value.get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect::<Vec<_>>())
}

fn print_contract_status(cwd: &Path) -> Result<()> {
    let domains = parse_toml_path(&bijux_dna_infra::configs_file(
        cwd,
        "ci/registry/domains.toml",
    ))?;
    let domain_rows = declared_toml_array(&domains, "domains")
        .ok_or_else(|| anyhow!("ci/registry/domains.toml must declare a domains array"))?;
    let images = parse_toml_path(&bijux_dna_infra::configs_file(cwd, "ci/tools/images.toml"))?;
    let image_ids = images
        .as_table()
        .ok_or_else(|| anyhow!("ci/tools/images.toml must contain a top-level table"))?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    println!(
        "{:<8} {:<12} {:<7} {:<7} {:<7} {:<7} {:<9} {:<8}",
        "domain", "mode", "stages", "params", "tools", "metrics", "images", "failures"
    );
    println!("{}", "-".repeat(74));

    for domain in domain_rows {
        let id = domain
            .get("id")
            .and_then(toml::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("invalid_domain");
        let experimental = domain
            .get("experimental")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        let stages_rel = declared_toml_str(domain, "stages_ssot");
        let params_rel = declared_toml_str(domain, "param_registry_ssot");
        let tools_rel = declared_toml_str(domain, "tool_registry_ssot");
        if stages_rel.is_none() || params_rel.is_none() || tools_rel.is_none() {
            println!(
                "{:<8} {:<12} {:<7} {:<7} {:<7} {:<7} {:<9} {:<8}",
                id, "invalid", "-", "-", "-", "-", "-", "yes"
            );
            continue;
        }
        let stages = parse_toml_path(&cwd.join(stages_rel.expect("checked above")))?;
        let params = parse_toml_path(&cwd.join(params_rel.expect("checked above")))?;
        let tools = parse_toml_path(&cwd.join(tools_rel.expect("checked above")))?;
        let stage_rows = declared_toml_array(&stages, "stages")
            .ok_or_else(|| anyhow!("stage registry must declare a stages array"))?;

        let param_stage_ids = param_rows(&params)?
            .into_iter()
            .filter_map(|row| row.get("stage_id").and_then(toml::Value::as_str))
            .collect::<BTreeSet<_>>();
        let tool_rows = declared_toml_array(&tools, "tools")
            .ok_or_else(|| anyhow!("tool registry must declare a tools array"))?;
        let mut tools_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
        let mut tool_metrics = BTreeMap::<String, String>::new();
        let mut invalid_tool_rows = 0usize;
        for row in &tool_rows {
            let Some(tool_id) = row
                .get("id")
                .and_then(toml::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned)
            else {
                invalid_tool_rows += 1;
                continue;
            };
            if let Some(metrics_schema) = declared_toml_str(row, "metrics_schema") {
                tool_metrics.insert(tool_id.clone(), metrics_schema.to_string());
            }
            for stage_id in toml_list(row, "stage_ids")? {
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
        let mut invalid_stage_rows = 0usize;
        for stage in stage_rows {
            let Some(stage_id) = stage
                .get("id")
                .and_then(toml::Value::as_str)
                .filter(|value| !value.trim().is_empty())
            else {
                invalid_stage_rows += 1;
                continue;
            };
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
            let stage_metrics_schema = declared_toml_str(stage, "metrics_schema");
            let stage_tools = toml_list(stage, "tools")?
                .into_iter()
                .chain(tools_by_stage.get(stage_id).into_iter().flatten().cloned())
                .collect::<BTreeSet<_>>();
            let has_metrics = stage_metrics_schema.is_some_and(|schema| schema == "none")
                || stage_metrics_schema.is_some()
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
        let failures = missing_params
            + missing_tools
            + missing_metrics
            + missing_images
            + invalid_tool_rows
            + invalid_stage_rows;
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
                let Some(stage_id) = parse_scalar(&raw, "stage_id") else {
                    missing_stage_fields.push(format!(
                        "{} missing required key `stage_id`",
                        path.display()
                    ));
                    continue;
                };
                let Some(status) = parse_scalar(&raw, "status") else {
                    missing_stage_fields.push(format!(
                        "{} missing required key `status`",
                        path.display()
                    ));
                    continue;
                };
                let Some(scope) = parse_scalar(&raw, "scope") else {
                    missing_stage_fields.push(format!(
                        "{} missing required key `scope`",
                        path.display()
                    ));
                    continue;
                };
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
                let Some(tool_id) = parse_scalar(&raw, "tool_id") else {
                    missing_tool_fields.push(format!(
                        "{} missing required key `tool_id`",
                        path.display()
                    ));
                    continue;
                };
                let Some(status) = parse_scalar(&raw, "status") else {
                    missing_tool_fields.push(format!(
                        "{} missing required key `status`",
                        path.display()
                    ));
                    continue;
                };
                let Some(scope) = parse_scalar(&raw, "scope") else {
                    missing_tool_fields.push(format!(
                        "{} missing required key `scope`",
                        path.display()
                    ));
                    continue;
                };
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
        let _ = writeln!(md, "- scope: `{}`", args.scope);
        let _ = writeln!(md, "- planned_or_out_of_scope: `{}`", planned.len());
        let _ = writeln!(md, "- placeholder_files: `{}`", placeholders.len());
        let _ = writeln!(md, "- missing_truth_fixtures: `{}`", missing_fixtures.len());
        let _ = writeln!(
            md,
            "- missing_stage_required_fields: `{}`",
            missing_stage_fields.len()
        );
        let _ = writeln!(
            md,
            "- missing_tool_required_fields: `{}`\n",
            missing_tool_fields.len()
        );

        md.push_str("## Planned / Out Of Scope\n");
        if planned.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &planned {
                let _ = writeln!(md, "- {item}");
            }
        }
        md.push_str("\n## Placeholder Files\n");
        if placeholders.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &placeholders {
                let _ = writeln!(md, "- {item}");
            }
        }
        md.push_str("\n## Missing Fixtures\n");
        if missing_fixtures.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_fixtures {
                let _ = writeln!(md, "- {item}");
            }
        }
        md.push_str("\n## Missing Stage Required Fields\n");
        if missing_stage_fields.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_stage_fields {
                let _ = writeln!(md, "- {item}");
            }
        }
        md.push_str("\n## Missing Tool Required Fields\n");
        if missing_tool_fields.is_empty() {
            md.push_str("- none\n");
        } else {
            for item in &missing_tool_fields {
                let _ = writeln!(md, "- {item}");
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

#[cfg(test)]
mod argv_normalization_contracts {
    use super::normalize_cli_argv;

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn direct_runtime_and_host_runtime_routes_normalize_to_the_same_surface() {
        let direct = normalize_cli_argv(&argv(&["bijux-dna", "registry", "list-stages"]));
        let host_route = normalize_cli_argv(&argv(&["bijux", "dna", "registry", "list-stages"]));
        let legacy_direct =
            normalize_cli_argv(&argv(&["bijux-dna", "dna", "registry", "list-stages"]));

        assert_eq!(direct, host_route);
        assert_eq!(direct, legacy_direct);
    }

    #[test]
    fn host_runtime_route_preserves_global_options_before_the_namespace() {
        let host_route = normalize_cli_argv(&argv(&[
            "bijux",
            "--json",
            "--platform",
            "test",
            "dna",
            "env",
            "info",
        ]));

        assert_eq!(
            host_route,
            argv(&["bijux-dna", "--json", "--platform", "test", "env", "info"])
        );
    }
}
