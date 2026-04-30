use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::{
    load_manifests, load_profile, resolve_run_base_dir, stage_external_asset_requirement,
};
use bijux_dna_api::v1::api::run::{CategorizedError, ErrorCategory};

use crate::commands::router::argv::parse_cli_from_argv;
use crate::commands::router::root::try_handle_root_command;
use crate::commands::router::runtime::{
    capture_cli_env, configure_process_cli_env, configure_run_context_env, enter_cli_cwd,
};
use crate::commands::{bam, bench, cli, fastq, planning, vcf};

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_args(args: &[&str], cwd: &Path) -> Result<()> {
    let argv_parts = args.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let cli = parse_cli_from_argv(&argv_parts)?;
    run_with_cli(&cli, cwd)
}

/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_with_cli(cli: &cli::Cli, cwd: &Path) -> Result<()> {
    let _guard = enter_cli_cwd(cwd)?;
    let _env_guard = capture_cli_env();
    configure_process_cli_env(cli, cwd);
    let domain_dir = cwd.join("domain");
    let registry_path = bijux_dna_infra::configs_file(cwd, "ci/registry/tool_registry.toml");

    let dna_command = &cli.command;
    if try_handle_root_command(dna_command, cwd, cli.platform.as_deref())? {
        return Ok(());
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
            matches!(dna_command, cli::DnaCommand::Bam(..) | cli::DnaCommand::Vcf(..))
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    } {
        let (stage, _, _) = cli::resolve_stage_tool(dna_command);
        enforce_offline_runtime_policy()?;
        ensure_stage_bank_requirements(cwd, stage.as_str())?;
        let run_domain = stage.as_str().split('.').next().unwrap_or("fastq").to_string();
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

    planning::run_plan(cli, dna_command, &registry, &domain_dir)
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
    let Some(requirement) = stage_external_asset_requirement(stage_id) else {
        return Ok(());
    };
    let mut candidates = Vec::new();
    if let Ok(hpc_root) =
        crate::commands::hpc::load_hpc_config().map(|cfg| cfg.resolve_paths().root)
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
            "stage `{stage_id}` requires a governed {} but no non-empty banks/assets directory was found",
            requirement.asset_class.label()
        ));
    }
    Ok(())
}

fn handle_observability_commands(dna_command: &cli::DnaCommand, cwd: &Path) -> Result<bool> {
    #[cfg(not(debug_assertions))]
    let _ = cwd;

    match dna_command {
        #[cfg(debug_assertions)]
        cli::DnaCommand::Debug(args) => {
            if args.view != "tail" {
                return Err(anyhow!("unsupported --view `{}` (expected `tail`)", args.view));
            }
            let run_dir = observability_run_dir(cwd, &args.search_root, &args.run_id)?;
            let telemetry_path = run_dir.join("run_artifacts").join("telemetry.jsonl");
            let raw = std::fs::read_to_string(&telemetry_path)
                .with_context(|| format!("read {}", telemetry_path.display()))?;
            let mut failure: Option<serde_json::Value> = None;
            for line in raw.lines() {
                let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                if event.get("event_name").and_then(serde_json::Value::as_str) == Some("run_failed")
                {
                    failure = Some(event);
                }
            }
            if let Some(event) = failure {
                println!("run_id: {}", json_string(&event, "run_id"));
                println!("stage_id: {}", json_string(&event, "stage_id"));
                println!("tool_id: {}", json_string(&event, "tool_id"));
                println!("status: {}", json_string(&event, "status"));
                if let Some(code) = event.get("failure_code") {
                    println!("failure_code: {}", serde_json::to_string(code)?);
                }
                if let Some(stderr) = event.get("attrs").and_then(|attrs| attrs.get("stderr_path"))
                {
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
            let run_dir = observability_run_dir(cwd, &args.search_root, &args.run)?;
            let out = args
                .out
                .clone()
                .unwrap_or_else(|| run_dir.join(format!("{}-log-pack.tar", args.run)));
            write_observability_log_pack(&run_dir, &out)?;
            println!("{}", out.display());
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value.get(key).and_then(serde_json::Value::as_str).unwrap_or("unknown").to_string()
}

const OBSERVABILITY_LOG_PACK_FILES: [&str; 4] = [
    "run_manifest.json",
    "run_manifest.lock.json",
    "run_artifacts/telemetry.jsonl",
    "logs/stderr.log",
];

fn existing_observability_log_pack_files(run_dir: &Path) -> Vec<&'static str> {
    OBSERVABILITY_LOG_PACK_FILES.into_iter().filter(|rel| run_dir.join(rel).is_file()).collect()
}

fn write_observability_log_pack(run_dir: &Path, out: &Path) -> Result<()> {
    let entries = existing_observability_log_pack_files(run_dir);
    if entries.is_empty() {
        return Err(anyhow!("no observability artifacts found in {}", run_dir.display()));
    }

    let file =
        bijux_dna_infra::create_file(out).with_context(|| format!("create {}", out.display()))?;
    let mut archive = tar::Builder::new(file);
    for rel in entries {
        let path = run_dir.join(rel);
        append_observability_log_entry(&mut archive, &path, rel)?;
    }
    archive.finish().context("finalize log-pack tar")?;
    Ok(())
}

fn append_observability_log_entry(
    archive: &mut tar::Builder<std::fs::File>,
    path: &Path,
    rel: &str,
) -> Result<()> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    archive
        .append_data(&mut header, rel, bytes.as_slice())
        .with_context(|| format!("add {} to log-pack", path.display()))?;
    Ok(())
}

fn observability_run_dir(cwd: &Path, search_root: &Path, run_id: &str) -> Result<PathBuf> {
    ensure_observability_run_id(run_id)?;
    Ok(cwd.join(search_root).join(run_id))
}

fn ensure_observability_run_id(run_id: &str) -> Result<()> {
    if run_id.trim().is_empty() {
        return Err(anyhow!("observability run id must be non-empty"));
    }
    let mut components = Path::new(run_id).components();
    let Some(Component::Normal(_)) = components.next() else {
        return Err(anyhow!("observability run id must be a single path segment"));
    };
    if components.next().is_some() {
        return Err(anyhow!("observability run id must be a single path segment"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{observability_run_dir, write_observability_log_pack};

    #[test]
    fn observability_run_dir_accepts_single_segment_run_ids() {
        let run_dir = observability_run_dir(
            std::path::Path::new("/workspace"),
            std::path::Path::new("artifacts/bench"),
            "run-001",
        )
        .expect("run dir");

        assert_eq!(run_dir, std::path::Path::new("/workspace/artifacts/bench/run-001"));
    }

    #[test]
    fn observability_run_dir_rejects_path_like_run_ids() {
        for run_id in ["", "../run-001", "nested/run-001", "/tmp/run-001", "."] {
            let err = observability_run_dir(
                std::path::Path::new("/workspace"),
                std::path::Path::new("artifacts/bench"),
                run_id,
            )
            .expect_err("run id should be rejected");

            assert!(err.to_string().contains("observability run id"));
        }
    }

    #[test]
    fn write_observability_log_pack_rejects_empty_runs_without_creating_archive() {
        let temp = tempfile::tempdir().expect("tempdir");
        let run_dir = temp.path().join("run-001");
        let out = temp.path().join("run-001.tar");
        bijux_dna_infra::ensure_dir(&run_dir).expect("run dir");

        let err = write_observability_log_pack(&run_dir, &out).expect_err("empty pack should fail");

        assert!(err.to_string().contains("no observability artifacts found"));
        assert!(!out.exists());
    }

    #[test]
    fn write_observability_log_pack_is_stable_across_file_metadata_changes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let run_dir = temp.path().join("run-001");
        let manifest = run_dir.join("run_manifest.json");
        let first_out = temp.path().join("first.tar");
        let second_out = temp.path().join("second.tar");
        bijux_dna_infra::ensure_dir(&run_dir).expect("run dir");
        bijux_dna_infra::write_bytes(&manifest, b"{\"run\":\"run-001\"}\n").expect("manifest");

        filetime::set_file_mtime(&manifest, filetime::FileTime::from_unix_time(10, 0))
            .expect("old mtime");
        write_observability_log_pack(&run_dir, &first_out).expect("first pack");

        filetime::set_file_mtime(&manifest, filetime::FileTime::from_unix_time(20, 0))
            .expect("new mtime");
        write_observability_log_pack(&run_dir, &second_out).expect("second pack");

        let first = std::fs::read(first_out).expect("read first pack");
        let second = std::fs::read(second_out).expect("read second pack");
        assert_eq!(first, second);
    }
}
