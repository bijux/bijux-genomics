use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_retained_tools::{
    collect_all_domain_retained_tool_rows, AllDomainRetainedToolRow,
};
use super::tool_execution_modes::load_runtime_probe_with_source;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_EXECUTABLE_RESOLUTION_PATH: &str =
    "benchmarks/readiness/tools/executable-resolution.tsv";
const EXECUTABLE_RESOLUTION_SCHEMA_VERSION: &str = "bijux.bench.readiness.executable_resolution.v1";
const RESOLUTION_KIND_HOST_BINARY: &str = "host_binary";
const RESOLUTION_KIND_DOCKER_IMAGE: &str = "docker_image";
const RESOLUTION_KIND_APPTAINER_IMAGE: &str = "apptainer_image";
const RESOLUTION_KIND_UNAVAILABLE: &str = "unavailable_with_reason";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ExecutableResolutionRow {
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) active_stage_ids: Vec<String>,
    pub(crate) install_kind: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExecutableResolutionReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) resolution_counts: BTreeMap<String, usize>,
    pub(crate) unavailable_count: usize,
    pub(crate) rows: Vec<ExecutableResolutionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutableResolutionSignature {
    install_kind: String,
    resolution_kind: String,
    resolution_target: String,
    command_entrypoint: Option<String>,
    unavailable_reason: Option<String>,
}

pub(crate) fn run_render_executable_resolution(
    args: &parse::BenchReadinessRenderExecutableResolutionArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_executable_resolution(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_EXECUTABLE_RESOLUTION_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_executable_resolution(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ExecutableResolutionReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_executable_resolution_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_executable_resolution_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut resolution_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *resolution_counts.entry(row.resolution_kind.clone()).or_default() += 1;
    }
    let unavailable_count =
        resolution_counts.get(RESOLUTION_KIND_UNAVAILABLE).copied().unwrap_or_default();

    Ok(ExecutableResolutionReport {
        schema_version: EXECUTABLE_RESOLUTION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        resolution_counts,
        unavailable_count,
        rows,
    })
}

pub(crate) fn collect_executable_resolution_rows(
    repo_root: &Path,
) -> Result<Vec<ExecutableResolutionRow>> {
    let retained_rows = collect_all_domain_retained_tool_rows(repo_root)?;
    let mut rows = retained_rows
        .iter()
        .map(|row| resolve_executable_resolution_row(repo_root, row))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_executable_resolution_contract(&retained_rows, &rows)?;
    Ok(rows)
}

fn resolve_executable_resolution_row(
    repo_root: &Path,
    retained_row: &AllDomainRetainedToolRow,
) -> Result<ExecutableResolutionRow> {
    let mut runtime_probe_paths = Vec::<String>::new();
    let mut resolved_signature = None::<ExecutableResolutionSignature>;

    for domain in &retained_row.domains {
        let loaded = load_runtime_probe_with_source(repo_root, domain, &retained_row.tool_id)
            .with_context(|| {
                format!(
                    "load runtime probe for retained tool `{}` in domain `{}`",
                    retained_row.tool_id, domain
                )
            })?;
        let signature = resolve_runtime_probe_signature(&retained_row.tool_id, &loaded.probe);
        let probe_path = path_relative_to_repo(repo_root, &loaded.path);
        if !runtime_probe_paths.iter().any(|existing| existing == &probe_path) {
            runtime_probe_paths.push(probe_path);
        }

        if let Some(previous) = &resolved_signature {
            if previous != &signature {
                return Err(anyhow!(
                    "retained tool `{}` resolves inconsistently across governed domain runtime probes",
                    retained_row.tool_id
                ));
            }
        } else {
            resolved_signature = Some(signature);
        }
    }

    let signature = resolved_signature.ok_or_else(|| {
        anyhow!(
            "retained tool `{}` must resolve at least one governed runtime probe",
            retained_row.tool_id
        )
    })?;

    runtime_probe_paths.sort();

    Ok(ExecutableResolutionRow {
        tool_id: retained_row.tool_id.clone(),
        domains: retained_row.domains.clone(),
        active_stage_ids: retained_row.active_stage_ids.clone(),
        install_kind: signature.install_kind,
        resolution_kind: signature.resolution_kind,
        resolution_target: signature.resolution_target,
        command_entrypoint: signature.command_entrypoint,
        runtime_probe_paths,
        unavailable_reason: signature.unavailable_reason,
    })
}

fn resolve_runtime_probe_signature(
    tool_id: &str,
    probe: &super::tool_execution_modes::RuntimeProbe,
) -> ExecutableResolutionSignature {
    let install_kind = probe.install_kind().to_string();
    let command_entrypoint = probe.command_entrypoint();
    let container_id = probe.container_id();

    match install_kind.clone().as_str() {
        "workspace_binary" | "host_binary" => ExecutableResolutionSignature {
            install_kind,
            resolution_kind: RESOLUTION_KIND_HOST_BINARY.to_string(),
            resolution_target: command_entrypoint.clone().unwrap_or_else(|| tool_id.to_string()),
            command_entrypoint,
            unavailable_reason: None,
        },
        "container" => match container_id {
            Some(container_id) if is_external_reference(&container_id) => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares an external container source without a governed local image"
                        .to_string(),
                ),
            },
            Some(container_id) if is_planned_reference(&container_id) => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares a planned container source without a governed local image"
                        .to_string(),
                ),
            },
            Some(container_id) if is_apptainer_reference(&container_id) => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
                resolution_target: container_id,
                command_entrypoint,
                unavailable_reason: None,
            },
            Some(container_id) if is_docker_reference(&container_id) => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_DOCKER_IMAGE.to_string(),
                resolution_target: container_id,
                command_entrypoint,
                unavailable_reason: None,
            },
            Some(container_id) => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(format!(
                    "runtime probe declares unsupported container reference `{container_id}`"
                )),
            },
            None => ExecutableResolutionSignature {
                install_kind,
                resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
                resolution_target: String::new(),
                command_entrypoint,
                unavailable_reason: Some(
                    "runtime probe declares container install_kind without a governed image reference"
                        .to_string(),
                ),
            },
        },
        unsupported => ExecutableResolutionSignature {
            install_kind,
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            command_entrypoint,
            unavailable_reason: Some(format!(
                "runtime probe declares unsupported install_kind `{unsupported}`"
            )),
        },
    }
}

fn is_apptainer_reference(container_id: &str) -> bool {
    container_id.contains("/apptainer/")
        || container_id.ends_with(".def")
        || container_id.ends_with(".sif")
        || container_id.contains("/apptainer-")
}

fn is_external_reference(container_id: &str) -> bool {
    container_id == "external"
        || container_id.starts_with("external@")
        || container_id.ends_with("@external")
}

fn is_planned_reference(container_id: &str) -> bool {
    let trimmed = container_id.trim();
    trimmed == "planned"
        || trimmed == "pending"
        || trimmed.ends_with("@planned")
        || trimmed.ends_with("@pending")
}

fn is_docker_reference(container_id: &str) -> bool {
    container_id.contains("/docker/")
        || container_id.starts_with("docker.io/")
        || container_id.starts_with("ghcr.io/")
        || container_id.starts_with("quay.io/")
        || container_id.starts_with("bijuxdna/")
        || (!container_id.contains('/')
            && !container_id.ends_with(".def")
            && !container_id.ends_with(".sif"))
}

fn ensure_executable_resolution_contract(
    retained_rows: &[AllDomainRetainedToolRow],
    rows: &[ExecutableResolutionRow],
) -> Result<()> {
    if rows.len() != retained_rows.len() {
        return Err(anyhow!("executable resolution must keep exactly one row per retained tool"));
    }

    let retained_tool_ids = retained_rows
        .iter()
        .map(|row| row.tool_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let resolved_tool_ids =
        rows.iter().map(|row| row.tool_id.as_str()).collect::<std::collections::BTreeSet<_>>();
    if retained_tool_ids != resolved_tool_ids {
        return Err(anyhow!(
            "executable resolution drifted from the governed retained tool inventory"
        ));
    }

    for row in rows {
        if row.tool_id.trim().is_empty()
            || row.domains.is_empty()
            || row.active_stage_ids.is_empty()
            || row.install_kind.trim().is_empty()
            || row.resolution_kind.trim().is_empty()
            || row.runtime_probe_paths.is_empty()
        {
            return Err(anyhow!(
                "executable resolution row `{}` is missing a required field",
                row.tool_id
            ));
        }
        match row.resolution_kind.as_str() {
            RESOLUTION_KIND_HOST_BINARY
            | RESOLUTION_KIND_DOCKER_IMAGE
            | RESOLUTION_KIND_APPTAINER_IMAGE => {
                if row.resolution_target.trim().is_empty() || row.unavailable_reason.is_some() {
                    return Err(anyhow!(
                        "resolved tool `{}` must keep a concrete target and no unavailable reason",
                        row.tool_id
                    ));
                }
            }
            RESOLUTION_KIND_UNAVAILABLE => {
                if row.unavailable_reason.as_deref().map(str::trim).is_none_or(str::is_empty) {
                    return Err(anyhow!(
                        "unavailable tool `{}` must keep an explicit unavailable reason",
                        row.tool_id
                    ));
                }
                if !row.resolution_target.is_empty() {
                    return Err(anyhow!(
                        "unavailable tool `{}` must not keep a concrete resolution target",
                        row.tool_id
                    ));
                }
            }
            _ => {
                return Err(anyhow!(
                    "tool `{}` resolved to unsupported kind `{}`",
                    row.tool_id,
                    row.resolution_kind
                ));
            }
        }
    }

    Ok(())
}

fn render_executable_resolution_tsv(rows: &[ExecutableResolutionRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tdomains\tactive_stage_ids\tinstall_kind\tresolution_kind\tresolution_target\tcommand_entrypoint\truntime_probe_paths\tunavailable_reason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.tool_id,
            row.domains.join(","),
            row.active_stage_ids.join(","),
            row.install_kind,
            row.resolution_kind,
            row.resolution_target,
            row.command_entrypoint.clone().unwrap_or_default(),
            row.runtime_probe_paths.join(","),
            row.unavailable_reason.clone().unwrap_or_default(),
        ));
    }
    rendered
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
