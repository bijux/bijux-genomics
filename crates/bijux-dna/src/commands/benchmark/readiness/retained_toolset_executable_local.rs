use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::all_domain_retained_tools::{
    render_all_domain_retained_tools, DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH,
};
use super::apptainer_map::{render_apptainer_map, DEFAULT_APPTAINER_MAP_PATH};
use super::container_tool_probe::DEFAULT_CONTAINER_TOOL_SMOKE_ROOT;
use super::executable_resolution::{
    render_executable_resolution, DEFAULT_EXECUTABLE_RESOLUTION_PATH,
};
use super::host_tool_probe::DEFAULT_HOST_TOOL_SMOKE_ROOT;
use super::input_preflight_audit::{
    render_input_preflight_audit, DEFAULT_INPUT_PREFLIGHT_TESTS_PATH,
};
use super::output_contract_audit::{
    render_output_contract_audit, DEFAULT_OUTPUT_CONTRACT_TESTS_PATH,
};
use super::real_output_parser_probe::{
    render_real_output_parser_smoke, DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH,
};
use super::version_probes::{
    collect_version_probe_rows, render_version_probes, DEFAULT_VERSION_PROBES_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_RETAINED_TOOLSET_EXECUTABLE_LOCAL_PATH: &str =
    "benchmarks/readiness/tools/RETAINED_TOOLSET_EXECUTABLE_LOCAL.json";
const RETAINED_TOOLSET_EXECUTABLE_LOCAL_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.retained_toolset_executable_local.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RetainedToolsetExecutableLocalGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RetainedToolsetExecutableLocalReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) retained_tool_count: usize,
    pub(crate) executable_resolution_row_count: usize,
    pub(crate) version_probe_row_count: usize,
    pub(crate) host_smoke_tool_count: usize,
    pub(crate) container_smoke_tool_count: usize,
    pub(crate) parser_family_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<RetainedToolsetExecutableLocalGoalCheck>,
}

pub(crate) fn run_render_retained_toolset_executable_local(
    args: &parse::BenchReadinessRenderRetainedToolsetExecutableLocalArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_retained_toolset_executable_local(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_RETAINED_TOOLSET_EXECUTABLE_LOCAL_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_retained_toolset_executable_local(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RetainedToolsetExecutableLocalReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let mut retained_tool_count = 0usize;
    let mut executable_resolution_row_count = 0usize;
    let mut version_probe_row_count = 0usize;
    let mut host_smoke_tool_count = 0usize;
    let mut container_smoke_tool_count = 0usize;
    let mut parser_family_count = 0usize;

    record_goal_check(
        &mut checks,
        421,
        "retained_tool_inventory",
        Some(DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH.to_string()),
        || {
            let report = render_all_domain_retained_tools(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH),
            )?;
            retained_tool_count = report.row_count;
            Ok(format!(
                "row_count={}, benchmark_ready_tool_count={}",
                report.row_count, report.benchmark_ready_tool_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        422,
        "retained_tool_executable_resolution",
        Some(DEFAULT_EXECUTABLE_RESOLUTION_PATH.to_string()),
        || {
            let report = render_executable_resolution(
                repo_root,
                PathBuf::from(DEFAULT_EXECUTABLE_RESOLUTION_PATH),
            )?;
            executable_resolution_row_count = report.row_count;
            let host_binary_count =
                report.resolution_counts.get("host_binary").copied().unwrap_or_default();
            let container_count = report
                .resolution_counts
                .iter()
                .filter(|(kind, _)| matches!(kind.as_str(), "docker_image" | "apptainer_image"))
                .map(|(_, count)| *count)
                .sum::<usize>();
            Ok(format!(
                "row_count={}, host_binary_count={}, container_count={}, unavailable_count={}",
                report.row_count, host_binary_count, container_count, report.unavailable_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        423,
        "retained_tool_version_probes",
        Some(DEFAULT_VERSION_PROBES_PATH.to_string()),
        || {
            let report =
                render_version_probes(repo_root, PathBuf::from(DEFAULT_VERSION_PROBES_PATH))?;
            version_probe_row_count = report.row_count;
            Ok(format!(
                "row_count={}, ready_count={}, unavailable_count={}",
                report.row_count, report.ready_count, report.unavailable_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        424,
        "retained_tool_host_smoke",
        Some(DEFAULT_HOST_TOOL_SMOKE_ROOT.to_string()),
        || {
            let summary = validate_host_tool_smoke_manifests(repo_root)?;
            host_smoke_tool_count = summary.tool_count;
            Ok(format!(
                "tool_count={}, success_count={}",
                summary.tool_count, summary.success_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        425,
        "retained_tool_container_smoke",
        Some(DEFAULT_CONTAINER_TOOL_SMOKE_ROOT.to_string()),
        || {
            let summary = validate_container_tool_smoke_manifests(repo_root)?;
            container_smoke_tool_count = summary.tool_count;
            Ok(format!(
                "tool_count={}, success_count={}, unavailable_count={}, failure_count={}",
                summary.tool_count,
                summary.success_count,
                summary.unavailable_count,
                summary.failure_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        426,
        "retained_tool_apptainer_map",
        Some(DEFAULT_APPTAINER_MAP_PATH.to_string()),
        || {
            let report =
                render_apptainer_map(repo_root, PathBuf::from(DEFAULT_APPTAINER_MAP_PATH))?;
            Ok(format!(
                "row_count={}, docker_runtime={}, covered_domain_count={}",
                report.row_count,
                report.docker_runtime,
                report.domain_counts.len()
            ))
        },
    );
    record_goal_check(
        &mut checks,
        427,
        "retained_tool_input_preflight",
        Some(DEFAULT_INPUT_PREFLIGHT_TESTS_PATH.to_string()),
        || {
            let report = render_input_preflight_audit(
                repo_root,
                PathBuf::from(DEFAULT_INPUT_PREFLIGHT_TESTS_PATH),
            )?;
            Ok(format!(
                "row_count={}, passed_row_count={}, covered_tool_count={}",
                report.row_count, report.passed_row_count, report.covered_tool_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        428,
        "retained_tool_output_contracts",
        Some(DEFAULT_OUTPUT_CONTRACT_TESTS_PATH.to_string()),
        || {
            let report = render_output_contract_audit(
                repo_root,
                PathBuf::from(DEFAULT_OUTPUT_CONTRACT_TESTS_PATH),
            )?;
            Ok(format!(
                "row_count={}, passed_row_count={}, failed_row_count={}",
                report.row_count, report.passed_row_count, report.failed_row_count
            ))
        },
    );
    record_goal_check(
        &mut checks,
        429,
        "retained_tool_real_output_parser_smoke",
        Some(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH.to_string()),
        || {
            let report = render_real_output_parser_smoke(
                repo_root,
                PathBuf::from(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH),
            )?;
            parser_family_count = report.family_count;
            Ok(format!(
                "family_count={}, passed_family_count={}, failed_family_count={}",
                report.family_count, report.passed_family_count, report.failed_family_count
            ))
        },
    );

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let checked_goal_count = checks.len();
    let failed_goal_count = checked_goal_count.saturating_sub(passed_goal_count);
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect();
    let report = RetainedToolsetExecutableLocalReport {
        schema_version: RETAINED_TOOLSET_EXECUTABLE_LOCAL_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        retained_tool_count,
        executable_resolution_row_count,
        version_probe_row_count,
        host_smoke_tool_count,
        container_smoke_tool_count,
        parser_family_count,
        ok: failed_goal_count == 0,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn record_goal_check<F>(
    checks: &mut Vec<RetainedToolsetExecutableLocalGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(RetainedToolsetExecutableLocalGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(RetainedToolsetExecutableLocalGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HostSmokeSummary {
    tool_count: usize,
    success_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContainerSmokeSummary {
    tool_count: usize,
    success_count: usize,
    unavailable_count: usize,
    failure_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ValidatedHostToolSmokeManifest {
    tool_id: String,
    status: String,
    exit_code: i32,
    version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ValidatedContainerToolSmokeManifest {
    tool_id: String,
    status: String,
    exit_code: Option<i32>,
    unavailable_reason: Option<String>,
}

fn validate_host_tool_smoke_manifests(repo_root: &Path) -> Result<HostSmokeSummary> {
    let host_rows = collect_version_probe_rows(repo_root)?
        .into_iter()
        .filter(|row| row.resolution_kind == "host_binary" && row.version_probe_status == "ready")
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    for row in &host_rows {
        let manifest_path =
            repo_root.join(DEFAULT_HOST_TOOL_SMOKE_ROOT).join(&row.tool_id).join("manifest.json");
        let manifest: ValidatedHostToolSmokeManifest = read_json_document(&manifest_path)?;
        if manifest.tool_id != row.tool_id {
            failures.push(format!(
                "{}: manifest tool_id `{}` does not match expected `{}`",
                manifest_path.display(),
                manifest.tool_id,
                row.tool_id
            ));
        }
        if manifest.status != "ok" {
            failures.push(format!(
                "{}: expected status `ok`, found `{}`",
                manifest_path.display(),
                manifest.status
            ));
        }
        if manifest.exit_code != 0 {
            failures.push(format!(
                "{}: expected exit_code 0, found {}",
                manifest_path.display(),
                manifest.exit_code
            ));
        }
        if manifest.version.as_deref().is_none_or(str::is_empty) {
            failures.push(format!(
                "{}: missing parsed version for host smoke manifest",
                manifest_path.display()
            ));
        }
    }
    if !failures.is_empty() {
        return Err(anyhow!(
            "retained host smoke manifests failed validation: {}",
            failures.join(" | ")
        ));
    }
    Ok(HostSmokeSummary { tool_count: host_rows.len(), success_count: host_rows.len() })
}

fn validate_container_tool_smoke_manifests(repo_root: &Path) -> Result<ContainerSmokeSummary> {
    let container_rows = collect_version_probe_rows(repo_root)?
        .into_iter()
        .filter(|row| row.resolution_kind != "host_binary")
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    let mut success_count = 0usize;
    let mut unavailable_count = 0usize;
    for row in &container_rows {
        let manifest_path = repo_root
            .join(DEFAULT_CONTAINER_TOOL_SMOKE_ROOT)
            .join(&row.tool_id)
            .join("manifest.json");
        let manifest: ValidatedContainerToolSmokeManifest = read_json_document(&manifest_path)?;
        if manifest.tool_id != row.tool_id {
            failures.push(format!(
                "{}: manifest tool_id `{}` does not match expected `{}`",
                manifest_path.display(),
                manifest.tool_id,
                row.tool_id
            ));
            continue;
        }
        match row.version_probe_status.as_str() {
            "ready" => {
                if manifest.status != "ok" {
                    failures.push(format!(
                        "{}: expected status `ok`, found `{}`",
                        manifest_path.display(),
                        manifest.status
                    ));
                    continue;
                }
                if manifest.exit_code != Some(0) {
                    failures.push(format!(
                        "{}: expected exit_code 0, found {:?}",
                        manifest_path.display(),
                        manifest.exit_code
                    ));
                    continue;
                }
                success_count += 1;
            }
            "unavailable_with_reason" => {
                if manifest.status != "unavailable_with_reason" {
                    failures.push(format!(
                        "{}: expected status `unavailable_with_reason`, found `{}`",
                        manifest_path.display(),
                        manifest.status
                    ));
                    continue;
                }
                if manifest.unavailable_reason.as_deref().is_none_or(str::is_empty) {
                    failures.push(format!(
                        "{}: unavailable manifest is missing unavailable_reason",
                        manifest_path.display()
                    ));
                    continue;
                }
                unavailable_count += 1;
            }
            other => failures.push(format!(
                "{}: unsupported version probe status `{other}` for `{}`",
                manifest_path.display(),
                row.tool_id
            )),
        }
    }
    if !failures.is_empty() {
        return Err(anyhow!(
            "retained container smoke manifests failed validation: {}",
            failures.join(" | ")
        ));
    }
    Ok(ContainerSmokeSummary {
        tool_count: container_rows.len(),
        success_count,
        unavailable_count,
        failure_count: 0,
    })
}

fn read_json_document<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
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
