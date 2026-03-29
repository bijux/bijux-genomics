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
use crate::commands::status::handle_status_root;
use crate::commands::{bam, bench, cli, fastq, run_plan, vcf};

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
