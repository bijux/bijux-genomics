use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Serialize;

use super::version_probes::{collect_version_probe_rows, VersionProbeRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_HOST_TOOL_SMOKE_ROOT: &str = "runs/bench/tool-smoke/host";
const HOST_TOOL_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bench.host_tool_smoke_report.v1";
const HOST_TOOL_SMOKE_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.host_tool_smoke_manifest.v1";
const HOST_TOOL_SMOKE_STATUS_OK: &str = "ok";
const HOST_TOOL_SMOKE_STATUS_COMMAND_FAILED: &str = "command_failed";
const HOST_TOOL_SMOKE_STATUS_VERSION_PARSE_FAILED: &str = "version_parse_failed";
const HOST_TOOL_SMOKE_STATUS_REGEX_MISMATCH: &str = "regex_mismatch";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HostToolSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_root: String,
    pub(crate) tool_count: usize,
    pub(crate) success_count: usize,
    pub(crate) failure_count: usize,
    pub(crate) rows: Vec<HostToolSmokeReportRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HostToolSmokeReportRow {
    pub(crate) tool_id: String,
    pub(crate) manifest_path: String,
    pub(crate) status: String,
    pub(crate) command: String,
    pub(crate) exit_code: i32,
    pub(crate) version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HostToolSmokeManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) tool_id: String,
    pub(crate) status: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) declared_command: String,
    pub(crate) applied_command: Vec<String>,
    pub(crate) working_directory: String,
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) version: Option<String>,
    pub(crate) version_parser_kind: String,
    pub(crate) expected_version_regex: String,
    pub(crate) version_matches_regex: bool,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) registry_paths: Vec<String>,
    pub(crate) checked_at_unix_s: u64,
}

pub(crate) fn run_host_tool_smoke(args: &parse::BenchReadinessRunHostToolSmokeArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_host_tool_smoke(
        &repo_root,
        args.output_root.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_HOST_TOOL_SMOKE_ROOT)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_root);
    }
    Ok(())
}

pub(crate) fn render_host_tool_smoke(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<HostToolSmokeReport> {
    let output_root = repo_relative_path(repo_root, &output_root);
    let host_rows = collect_host_version_probe_rows(repo_root)?;
    let mut report_rows = Vec::with_capacity(host_rows.len());
    let mut success_count = 0usize;

    for row in host_rows {
        let manifest = build_host_tool_smoke_manifest(repo_root, &row)?;
        let manifest_dir = output_root.join(&row.tool_id);
        fs::create_dir_all(&manifest_dir)
            .with_context(|| format!("create {}", manifest_dir.display()))?;
        let manifest_path = manifest_dir.join("manifest.json");
        bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)
            .with_context(|| format!("write {}", manifest_path.display()))?;
        if manifest.status == HOST_TOOL_SMOKE_STATUS_OK {
            success_count += 1;
        }
        report_rows.push(HostToolSmokeReportRow {
            tool_id: manifest.tool_id.clone(),
            manifest_path: path_relative_to_repo(repo_root, &manifest_path),
            status: manifest.status.clone(),
            command: manifest.declared_command.clone(),
            exit_code: manifest.exit_code,
            version: manifest.version.clone(),
        });
    }

    let failure_count = report_rows.len().saturating_sub(success_count);
    if failure_count > 0 {
        bail!("host tool smoke recorded {failure_count} failed command probes");
    }

    Ok(HostToolSmokeReport {
        schema_version: HOST_TOOL_SMOKE_REPORT_SCHEMA_VERSION,
        output_root: path_relative_to_repo(repo_root, &output_root),
        tool_count: report_rows.len(),
        success_count,
        failure_count,
        rows: report_rows,
    })
}

fn collect_host_version_probe_rows(repo_root: &Path) -> Result<Vec<VersionProbeRow>> {
    let rows = collect_version_probe_rows(repo_root)?;
    let mut host_rows =
        rows.into_iter().filter(|row| row.resolution_kind == "host_binary").collect::<Vec<_>>();
    host_rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    if host_rows.is_empty() {
        bail!("host tool smoke expected at least one governed host-binary retained tool");
    }
    for row in &host_rows {
        if row.version_probe_status != "ready" {
            bail!(
                "host tool `{}` must keep a ready governed version probe before smoke execution",
                row.tool_id
            );
        }
    }
    Ok(host_rows)
}

fn build_host_tool_smoke_manifest(
    repo_root: &Path,
    row: &VersionProbeRow,
) -> Result<HostToolSmokeManifest> {
    let declared_command = row.version_cmd.clone().ok_or_else(|| {
        anyhow!(
            "host tool `{}` is missing the governed version command required for smoke execution",
            row.tool_id
        )
    })?;
    let version_parser_kind = row.version_parser_kind.clone().ok_or_else(|| {
        anyhow!(
            "host tool `{}` is missing the governed version parser kind required for smoke execution",
            row.tool_id
        )
    })?;
    let expected_version_regex = row.expected_version_regex.clone().ok_or_else(|| {
        anyhow!(
            "host tool `{}` is missing the governed version regex required for smoke execution",
            row.tool_id
        )
    })?;
    if version_parser_kind != "first_dotted_numeric_token" {
        bail!(
            "host tool `{}` declares unsupported parser kind `{}`",
            row.tool_id,
            version_parser_kind
        );
    }
    let expected_version_regex = Regex::new(&expected_version_regex).with_context(|| {
        format!("host tool `{}` declares invalid expected_version_regex", row.tool_id)
    })?;

    let applied_command = resolve_host_command(repo_root, row, &declared_command)?;
    let output = run_host_command(repo_root, &applied_command)?;
    let stdout = String::from_utf8(output.stdout).context("decode stdout utf8")?;
    let stderr = String::from_utf8(output.stderr).context("decode stderr utf8")?;
    let combined_output = combined_output(&stdout, &stderr);
    let version = parse_first_version(&combined_output);
    let version_matches_regex = expected_version_regex.is_match(&combined_output);
    let exit_code = output.status.code().unwrap_or(-1);
    let status = if !output.status.success() {
        HOST_TOOL_SMOKE_STATUS_COMMAND_FAILED
    } else if version.is_none() {
        HOST_TOOL_SMOKE_STATUS_VERSION_PARSE_FAILED
    } else if !version_matches_regex {
        HOST_TOOL_SMOKE_STATUS_REGEX_MISMATCH
    } else {
        HOST_TOOL_SMOKE_STATUS_OK
    };

    Ok(HostToolSmokeManifest {
        schema_version: HOST_TOOL_SMOKE_MANIFEST_SCHEMA_VERSION,
        tool_id: row.tool_id.clone(),
        status: status.to_string(),
        resolution_kind: row.resolution_kind.clone(),
        resolution_target: row.resolution_target.clone(),
        command_entrypoint: row.command_entrypoint.clone(),
        declared_command,
        applied_command,
        working_directory: ".".to_string(),
        exit_code,
        stdout,
        stderr,
        version,
        version_parser_kind,
        expected_version_regex: expected_version_regex.as_str().to_string(),
        version_matches_regex,
        runtime_probe_paths: row.runtime_probe_paths.clone(),
        registry_paths: row.registry_paths.clone(),
        checked_at_unix_s: now_unix_s(),
    })
}

fn resolve_host_command(
    repo_root: &Path,
    row: &VersionProbeRow,
    declared_command: &str,
) -> Result<Vec<String>> {
    let current_exe = std::env::current_exe().context("resolve current executable")?;
    let current_exe_name = current_exe
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("current executable is missing a utf-8 file name"))?;
    let command_tokens =
        declared_command.split_whitespace().map(ToOwned::to_owned).collect::<Vec<_>>();
    if command_tokens.is_empty() {
        bail!("host tool `{}` declares an empty version command", row.tool_id);
    }

    if command_tokens[0] == row.resolution_target || command_tokens[0] == current_exe_name {
        let executable = path_relative_to_repo(repo_root, &current_exe);
        let mut applied = vec![executable];
        applied.extend(command_tokens.into_iter().skip(1));
        return Ok(applied);
    }

    Ok(vec!["sh".to_string(), "-lc".to_string(), declared_command.to_string()])
}

fn run_host_command(repo_root: &Path, applied_command: &[String]) -> Result<std::process::Output> {
    let Some(program) = applied_command.first() else {
        bail!("host smoke attempted to run an empty command");
    };
    let mut command = Command::new(program);
    command.args(applied_command.iter().skip(1));
    command.current_dir(repo_root);
    command
        .output()
        .with_context(|| format!("run host smoke command `{}`", applied_command.join(" ")))
}

fn combined_output(stdout: &str, stderr: &str) -> String {
    if stdout.is_empty() {
        stderr.to_string()
    } else if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    }
}

fn parse_first_version(output: &str) -> Option<String> {
    let mut chars = output.chars().peekable();
    let mut token = String::new();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            token.push(ch);
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() || *next == '.' || *next == '-' {
                    token.push(*next);
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            if token.contains('.') {
                return Some(token);
            }
            token.clear();
        }
    }
    None
}

fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
