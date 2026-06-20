use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_api::v1::api::run::run_command;
use serde::Serialize;

use super::tool_smoke_support::{
    now_unix_s, path_relative_to_repo, repo_relative_path, run_command_with_timeout_and_env,
    CommandExecution,
};
use super::version_probes::{collect_version_probe_rows, VersionProbeRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CONTAINER_TOOL_SMOKE_ROOT: &str = "runs/bench/tool-smoke/container";
pub(crate) const DEFAULT_CONTAINER_TOOL_SMOKE_TIMEOUT_SECONDS: u64 = 300;
const CONTAINER_TOOL_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.container_tool_smoke_report.v1";
const CONTAINER_TOOL_SMOKE_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.container_tool_smoke_manifest.v1";
const CONTAINER_TOOL_SMOKE_STATUS_OK: &str = "ok";
const CONTAINER_TOOL_SMOKE_STATUS_UNAVAILABLE: &str = "unavailable_with_reason";
const CONTAINER_TOOL_SMOKE_STATUS_COMMAND_FAILED: &str = "command_failed";
const CONTAINER_TOOL_SMOKE_STATUS_TIMED_OUT: &str = "timed_out";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ContainerToolSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_root: String,
    pub(crate) tool_count: usize,
    pub(crate) success_count: usize,
    pub(crate) unavailable_count: usize,
    pub(crate) failure_count: usize,
    pub(crate) rows: Vec<ContainerToolSmokeReportRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ContainerToolSmokeReportRow {
    pub(crate) tool_id: String,
    pub(crate) manifest_path: String,
    pub(crate) status: String,
    pub(crate) smoke_runtime: Option<String>,
    pub(crate) declared_command: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ContainerToolSmokeManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) active_stage_ids: Vec<String>,
    pub(crate) status: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) smoke_runtime: Option<String>,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) declared_command: Option<String>,
    pub(crate) applied_command: Vec<String>,
    pub(crate) version_cmd: Option<String>,
    pub(crate) help_cmd: Option<String>,
    pub(crate) working_directory: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) unavailable_reason: Option<String>,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) registry_paths: Vec<String>,
    pub(crate) checked_at_unix_s: u64,
}

#[derive(Debug, Clone)]
struct PreparedSmokeCommand {
    smoke_runtime: Option<String>,
    declared_command: Option<String>,
    applied_command: Vec<String>,
    applied_env: Vec<(String, String)>,
    unavailable_reason: Option<String>,
}

pub(crate) fn run_container_tool_smoke(
    args: &parse::BenchReadinessRunContainerToolSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_container_tool_smoke(
        &repo_root,
        args.output_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONTAINER_TOOL_SMOKE_ROOT)),
        args.tools.as_deref(),
        Duration::from_secs(args.timeout_seconds),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_root);
    }
    Ok(())
}

pub(crate) fn render_container_tool_smoke(
    repo_root: &Path,
    output_root: PathBuf,
    requested_tools: Option<&str>,
    timeout: Duration,
) -> Result<ContainerToolSmokeReport> {
    let rows = collect_container_version_probe_rows(repo_root, requested_tools)?;
    render_container_tool_smoke_with_executor(
        repo_root,
        output_root,
        &rows,
        timeout,
        |root, argv, envs, limit, label| {
            run_command_with_timeout_and_env(root, argv, envs, limit, label)
        },
    )
}

fn render_container_tool_smoke_with_executor<F>(
    repo_root: &Path,
    output_root: PathBuf,
    rows: &[VersionProbeRow],
    timeout: Duration,
    execute: F,
) -> Result<ContainerToolSmokeReport>
where
    F: Fn(&Path, &[String], &[(String, String)], Duration, &str) -> Result<CommandExecution>,
{
    let output_root = repo_relative_path(repo_root, &output_root);
    let mut report_rows = Vec::with_capacity(rows.len());
    let mut success_count = 0usize;
    let mut unavailable_count = 0usize;

    for row in rows {
        let manifest = build_container_tool_smoke_manifest(repo_root, row, timeout, &execute)?;
        let manifest_dir = output_root.join(&row.tool_id);
        bijux_dna_infra::ensure_dir(&manifest_dir)
            .with_context(|| format!("create {}", manifest_dir.display()))?;
        let manifest_path = manifest_dir.join("manifest.json");
        bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)
            .with_context(|| format!("write {}", manifest_path.display()))?;

        match manifest.status.as_str() {
            CONTAINER_TOOL_SMOKE_STATUS_OK => success_count += 1,
            CONTAINER_TOOL_SMOKE_STATUS_UNAVAILABLE => unavailable_count += 1,
            _ => {}
        }

        report_rows.push(ContainerToolSmokeReportRow {
            tool_id: manifest.tool_id.clone(),
            manifest_path: path_relative_to_repo(repo_root, &manifest_path),
            status: manifest.status.clone(),
            smoke_runtime: manifest.smoke_runtime.clone(),
            declared_command: manifest.declared_command.clone(),
            exit_code: manifest.exit_code,
            unavailable_reason: manifest.unavailable_reason.clone(),
        });
    }

    let failure_count = report_rows
        .iter()
        .filter(|row| {
            row.status != CONTAINER_TOOL_SMOKE_STATUS_OK
                && row.status != CONTAINER_TOOL_SMOKE_STATUS_UNAVAILABLE
        })
        .count();
    if failure_count > 0 {
        bail!("container tool smoke recorded {failure_count} failed execution probes");
    }

    Ok(ContainerToolSmokeReport {
        schema_version: CONTAINER_TOOL_SMOKE_REPORT_SCHEMA_VERSION,
        output_root: path_relative_to_repo(repo_root, &output_root),
        tool_count: report_rows.len(),
        success_count,
        unavailable_count,
        failure_count,
        rows: report_rows,
    })
}

fn collect_container_version_probe_rows(
    repo_root: &Path,
    requested_tools: Option<&str>,
) -> Result<Vec<VersionProbeRow>> {
    let mut rows = collect_version_probe_rows(repo_root)?
        .into_iter()
        .filter(|row| row.resolution_kind != "host_binary")
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let requested = parse_requested_tools(requested_tools);
    if !requested.is_empty() {
        rows.retain(|row| requested.contains(row.tool_id.as_str()));
        let found = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
        let missing = requested
            .iter()
            .filter(|tool_id| !found.contains(tool_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            bail!(
                "container tool smoke requested unknown or host-only retained tools: {}",
                missing.join(", ")
            );
        }
    }

    if rows.is_empty() {
        bail!("container tool smoke expected at least one governed non-host retained tool");
    }

    for row in &rows {
        if row.version_probe_status != "ready"
            && row.version_probe_status != "unavailable_with_reason"
        {
            bail!(
                "container tool `{}` must keep a governed version probe status before smoke execution",
                row.tool_id
            );
        }
    }

    Ok(rows)
}

fn build_container_tool_smoke_manifest<F>(
    repo_root: &Path,
    row: &VersionProbeRow,
    timeout: Duration,
    execute: &F,
) -> Result<ContainerToolSmokeManifest>
where
    F: Fn(&Path, &[String], &[(String, String)], Duration, &str) -> Result<CommandExecution>,
{
    let prepared = prepare_container_smoke_command(repo_root, row)?;
    let (status, exit_code, stdout, stderr, unavailable_reason) = if let Some(reason) =
        prepared.unavailable_reason.clone()
    {
        (
            CONTAINER_TOOL_SMOKE_STATUS_UNAVAILABLE.to_string(),
            None,
            String::new(),
            String::new(),
            Some(reason),
        )
    } else {
        let declared = prepared.declared_command.as_deref().ok_or_else(|| {
            anyhow!("container tool `{}` is missing a governed smoke command", row.tool_id)
        })?;
        let execution = execute(
            repo_root,
            &prepared.applied_command,
            &prepared.applied_env,
            timeout,
            declared,
        )?;
        let status = if execution.timed_out {
            CONTAINER_TOOL_SMOKE_STATUS_TIMED_OUT
        } else if execution.exit_code == 0 {
            CONTAINER_TOOL_SMOKE_STATUS_OK
        } else {
            CONTAINER_TOOL_SMOKE_STATUS_COMMAND_FAILED
        };
        (status.to_string(), Some(execution.exit_code), execution.stdout, execution.stderr, None)
    };

    Ok(ContainerToolSmokeManifest {
        schema_version: CONTAINER_TOOL_SMOKE_MANIFEST_SCHEMA_VERSION,
        tool_id: row.tool_id.clone(),
        domains: row.domains.clone(),
        active_stage_ids: row.active_stage_ids.clone(),
        status,
        resolution_kind: row.resolution_kind.clone(),
        resolution_target: row.resolution_target.clone(),
        smoke_runtime: prepared.smoke_runtime,
        command_entrypoint: row.command_entrypoint.clone(),
        declared_command: prepared.declared_command,
        applied_command: prepared.applied_command,
        version_cmd: row.version_cmd.clone(),
        help_cmd: row.help_cmd.clone(),
        working_directory: ".".to_string(),
        exit_code,
        stdout,
        stderr,
        unavailable_reason,
        runtime_probe_paths: row.runtime_probe_paths.clone(),
        registry_paths: row.registry_paths.clone(),
        checked_at_unix_s: now_unix_s(),
    })
}

fn prepare_container_smoke_command(
    repo_root: &Path,
    row: &VersionProbeRow,
) -> Result<PreparedSmokeCommand> {
    let Some(smoke_executable) = current_smoke_executable(repo_root)? else {
        bail!("container tool smoke could not resolve a governed smoke executable");
    };
    let bijux_bin_env = governed_bijux_bin_env(repo_root)?;

    let registry_path = row.registry_paths.first().map(String::as_str);

    match row.resolution_kind.as_str() {
        "docker_image" => {
            if !command_on_path("docker") {
                return Ok(PreparedSmokeCommand {
                    smoke_runtime: Some("docker-arm64".to_string()),
                    declared_command: Some(format!("bijux-dna env smoke docker-arm64 {}", row.tool_id)),
                    applied_command: smoke_command_argv(
                        &smoke_executable,
                        "docker-arm64",
                    ),
                    applied_env: smoke_command_env(
                        &row.tool_id,
                        bijux_bin_env.clone(),
                        registry_path,
                    ),
                    unavailable_reason: Some(
                        "governed smoke runtime `docker-arm64` requires docker on PATH".to_string(),
                    ),
                });
            }
            Ok(PreparedSmokeCommand {
                smoke_runtime: Some("docker-arm64".to_string()),
                declared_command: Some(format!("bijux-dna env smoke docker-arm64 {}", row.tool_id)),
                applied_command: smoke_command_argv(&smoke_executable, "docker-arm64"),
                applied_env: smoke_command_env(
                    &row.tool_id,
                    bijux_bin_env.clone(),
                    registry_path,
                ),
                unavailable_reason: None,
            })
        }
        "apptainer_image" => {
            if !command_on_path("apptainer") {
                return Ok(PreparedSmokeCommand {
                    smoke_runtime: Some("apptainer".to_string()),
                    declared_command: Some(format!("bijux-dna env smoke apptainer {}", row.tool_id)),
                    applied_command: smoke_command_argv(&smoke_executable, "apptainer"),
                    applied_env: smoke_command_env(
                        &row.tool_id,
                        bijux_bin_env.clone(),
                        registry_path,
                    ),
                    unavailable_reason: Some(
                        "governed smoke runtime `apptainer` is not available on PATH".to_string(),
                    ),
                });
            }
            Ok(PreparedSmokeCommand {
                smoke_runtime: Some("apptainer".to_string()),
                declared_command: Some(format!("bijux-dna env smoke apptainer {}", row.tool_id)),
                applied_command: smoke_command_argv(&smoke_executable, "apptainer"),
                applied_env: smoke_command_env(
                    &row.tool_id,
                    bijux_bin_env.clone(),
                    registry_path,
                ),
                unavailable_reason: None,
            })
        }
        "unavailable_with_reason" => Ok(PreparedSmokeCommand {
            smoke_runtime: None,
            declared_command: None,
            applied_command: Vec::new(),
            applied_env: Vec::new(),
            unavailable_reason: Some(row.unavailable_reason.clone().unwrap_or_else(|| {
                "retained tool is governed as unavailable for local container smoke".to_string()
            })),
        }),
        unsupported => bail!(
            "container tool smoke expected only non-host retained tools, found `{unsupported}` for `{}`",
            row.tool_id
        ),
    }
}

fn current_smoke_executable(repo_root: &Path) -> Result<Option<String>> {
    let dev_executable = repo_root.join("artifacts/rust/target/debug/bijux-dna-dev");
    if dev_executable.is_file() {
        return Ok(Some(path_relative_to_repo(repo_root, &dev_executable)));
    }
    let executable = std::env::current_exe().context("resolve current executable")?;
    Ok(Some(path_relative_to_repo(repo_root, &executable)))
}

fn governed_bijux_bin_env(repo_root: &Path) -> Result<Option<String>> {
    let built_bijux = repo_root.join("artifacts/rust/target/debug/bijux-dna");
    if built_bijux.is_file() {
        return Ok(Some(path_relative_to_repo(repo_root, &built_bijux)));
    }
    let executable = std::env::current_exe().context("resolve current executable")?;
    let executable_name =
        executable.file_name().and_then(|value| value.to_str()).unwrap_or_default();
    if executable_name == "bijux-dna" {
        return Ok(Some(path_relative_to_repo(repo_root, &executable)));
    }
    Ok(None)
}

fn smoke_command_argv(smoke_executable: &str, runtime: &str) -> Vec<String> {
    let smoke_command = match runtime {
        "docker-arm64" => "smoke-containers-docker-arm64",
        "apptainer" => "smoke-containers-apptainer",
        unsupported => unsupported,
    };
    vec![
        smoke_executable.to_string(),
        "containers".to_string(),
        "run".to_string(),
        smoke_command.to_string(),
    ]
}

fn smoke_command_env(
    tool_id: &str,
    bijux_bin_env: Option<String>,
    registry_path: Option<&str>,
) -> Vec<(String, String)> {
    let mut envs = vec![("TOOLS".to_string(), tool_id.to_string())];
    if let Some(bijux_bin_env) = bijux_bin_env {
        envs.push(("BIJUX_BIN".to_string(), bijux_bin_env));
    }
    if let Some(registry_path) = registry_path.map(str::trim).filter(|path| !path.is_empty()) {
        envs.push(("BIJUX_TOOL_REGISTRY_PATH".to_string(), registry_path.to_string()));
    }
    envs
}

fn command_on_path(program: &str) -> bool {
    run_command(program, &["--version".to_string()]).is_ok()
}

fn parse_requested_tools(requested_tools: Option<&str>) -> BTreeSet<String> {
    requested_tools
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_row(tool_id: &str, resolution_kind: &str) -> VersionProbeRow {
        VersionProbeRow {
            tool_id: tool_id.to_string(),
            domains: vec!["vcf".to_string()],
            active_stage_ids: vec!["vcf.test".to_string()],
            resolution_kind: resolution_kind.to_string(),
            resolution_target: "governed-target".to_string(),
            version_probe_status: if resolution_kind == "unavailable_with_reason" {
                "unavailable_with_reason".to_string()
            } else {
                "ready".to_string()
            },
            command_entrypoint: Some(tool_id.to_string()),
            version_cmd: Some(format!("{tool_id} --version")),
            help_cmd: Some(format!("{tool_id} --help")),
            version_parser_kind: Some("first_dotted_numeric_token".to_string()),
            expected_version_regex: Some("v?[0-9]+[.][0-9]+".to_string()),
            expected_bin: Some(tool_id.to_string()),
            declared_version: Some("1.0.0".to_string()),
            runtime_probe_paths: vec!["domain/vcf/tools/test.yaml".to_string()],
            registry_paths: vec!["configs/ci/registry/tool_registry_vcf.toml".to_string()],
            unavailable_reason: if resolution_kind == "unavailable_with_reason" {
                Some("governed external runtime".to_string())
            } else {
                None
            },
        }
    }

    #[test]
    fn requested_tool_parser_ignores_empty_segments() {
        let parsed = parse_requested_tools(Some(" adapterremoval , ,shapeit5 "));
        assert_eq!(
            parsed.into_iter().collect::<Vec<_>>(),
            vec!["adapterremoval".to_string(), "shapeit5".to_string()]
        );
    }

    #[test]
    fn render_container_tool_smoke_records_unavailable_and_success_rows() -> Result<()> {
        let repo_root = tempfile::tempdir()?;
        let dev_binary = repo_root.path().join("artifacts/rust/target/debug/bijux-dna-dev");
        let bijux_binary = repo_root.path().join("artifacts/rust/target/debug/bijux-dna");
        bijux_dna_infra::ensure_dir(dev_binary.parent().expect("dev binary parent"))?;
        bijux_dna_infra::write_bytes(&dev_binary, b"#!/bin/sh\nexit 0\n")?;
        bijux_dna_infra::write_bytes(&bijux_binary, b"#!/bin/sh\nexit 0\n")?;
        let output_root = PathBuf::from("runs/bench/tool-smoke/container");
        let rows = vec![
            sample_row("adapterremoval", "docker_image"),
            sample_row("shapeit5", "unavailable_with_reason"),
        ];

        let report = render_container_tool_smoke_with_executor(
            repo_root.path(),
            output_root,
            &rows,
            Duration::from_secs(5),
            |_, argv, envs, _, _| {
                assert_eq!(
                    argv,
                    &vec![
                        "artifacts/rust/target/debug/bijux-dna-dev".to_string(),
                        "containers".to_string(),
                        "run".to_string(),
                        "smoke-containers-docker-arm64".to_string(),
                    ]
                );
                assert_eq!(
                    envs,
                    &vec![
                        ("TOOLS".to_string(), "adapterremoval".to_string()),
                        (
                            "BIJUX_BIN".to_string(),
                            "artifacts/rust/target/debug/bijux-dna".to_string(),
                        ),
                        (
                            "BIJUX_TOOL_REGISTRY_PATH".to_string(),
                            "configs/ci/registry/tool_registry_vcf.toml".to_string(),
                        ),
                    ]
                );
                Ok(CommandExecution {
                    exit_code: 0,
                    stdout: "smoke ok\n".to_string(),
                    stderr: String::new(),
                    timed_out: false,
                })
            },
        )?;

        assert_eq!(report.tool_count, 2);
        assert_eq!(report.success_count, 1);
        assert_eq!(report.unavailable_count, 1);
        assert_eq!(report.failure_count, 0);
        assert!(repo_root
            .path()
            .join("runs/bench/tool-smoke/container/adapterremoval/manifest.json")
            .is_file());
        assert!(repo_root
            .path()
            .join("runs/bench/tool-smoke/container/shapeit5/manifest.json")
            .is_file());
        Ok(())
    }

    #[test]
    fn render_container_tool_smoke_fails_when_execution_fails() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let dev_binary = repo_root.path().join("artifacts/rust/target/debug/bijux-dna-dev");
        let bijux_binary = repo_root.path().join("artifacts/rust/target/debug/bijux-dna");
        bijux_dna_infra::ensure_dir(dev_binary.parent().expect("dev binary parent"))
            .expect("create dev binary parent");
        bijux_dna_infra::write_bytes(&dev_binary, b"#!/bin/sh\nexit 0\n").expect("seed dev binary");
        bijux_dna_infra::write_bytes(&bijux_binary, b"#!/bin/sh\nexit 0\n")
            .expect("seed bijux binary");
        let output_root = PathBuf::from("runs/bench/tool-smoke/container");
        let rows = vec![sample_row("adapterremoval", "docker_image")];

        let err = render_container_tool_smoke_with_executor(
            repo_root.path(),
            output_root,
            &rows,
            Duration::from_secs(5),
            |_, _, _, _, _| {
                Ok(CommandExecution {
                    exit_code: 19,
                    stdout: String::new(),
                    stderr: "boom".to_string(),
                    timed_out: false,
                })
            },
        )
        .expect_err("execution failure must bubble through report boundary");

        assert!(
            err.to_string().contains("container tool smoke recorded 1 failed execution probes"),
            "unexpected error: {err}"
        );
    }
}
