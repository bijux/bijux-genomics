use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_hpc_input_discovery::path_relative_to_repo;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use super::readiness::apptainer_map::{collect_apptainer_map_rows, ApptainerMapRow};
use super::readiness::executable_resolution::{
    resolve_runtime_probe_signature, RuntimeResolutionSignature, RESOLUTION_KIND_APPTAINER_IMAGE,
    RESOLUTION_KIND_DOCKER_IMAGE, RESOLUTION_KIND_HOST_BINARY, RESOLUTION_KIND_UNAVAILABLE,
};
use super::readiness::tool_execution_modes::{
    load_runtime_probe_with_source, load_tool_execution_mode_assignments,
    ToolExecutionModeAssignment, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_execution_resolver.v1";
pub(crate) const DEFAULT_HPC_EXECUTION_RESOLVER_PATH: &str =
    "runs/bench/hpc-dry-run/execution-resolver.tsv";
const UNCLASSIFIED_EXECUTION_MODE: &str = "unclassified";
const UNCLASSIFIED_EXECUTION_MODE_SUMMARY: &str =
    "Selected tool is outside the governed benchmark execution-mode map; the resolver falls back to governed runtime probe evidence.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcExecutionResolverRow {
    pub(crate) tool_id: String,
    pub(crate) lookup_tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) execution_mode_summary: String,
    pub(crate) domains: Vec<String>,
    pub(crate) selected_stage_ids: Vec<String>,
    pub(crate) selected_job_count: usize,
    pub(crate) selected_job_ids: Vec<String>,
    pub(crate) selected_result_ids: Vec<String>,
    pub(crate) selected_pipeline_ids: Vec<String>,
    pub(crate) selected_node_ids: Vec<String>,
    pub(crate) install_kind: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) registry_paths: Vec<String>,
    pub(crate) unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalHpcExecutionResolverReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) resolution_counts: BTreeMap<String, usize>,
    pub(crate) unclassified_tool_count: usize,
    pub(crate) rows: Vec<LocalHpcExecutionResolverRow>,
}

#[derive(Debug, Clone, Default)]
struct SelectedToolScopeAccumulator {
    domains: BTreeSet<String>,
    selected_stage_ids: BTreeSet<String>,
    selected_job_ids: BTreeSet<String>,
    selected_result_ids: BTreeSet<String>,
    selected_pipeline_ids: BTreeSet<String>,
    selected_node_ids: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct SelectedToolScope {
    tool_id: String,
    lookup_tool_id: String,
    domains: Vec<String>,
    selected_stage_ids: Vec<String>,
    selected_job_ids: Vec<String>,
    selected_result_ids: Vec<String>,
    selected_pipeline_ids: Vec<String>,
    selected_node_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResolvedRuntimeProbeObservation {
    signature: RuntimeResolutionSignature,
    runtime_probe_paths: Vec<String>,
}

pub(crate) fn run_render_hpc_execution_resolver(
    args: &parse::BenchLocalRenderHpcExecutionResolverArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("execution-resolver.tsv")
    });
    let report = render_hpc_execution_resolver(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_execution_resolver(
    args: &parse::BenchLocalValidateHpcExecutionResolverArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("execution-resolver.tsv")
    });
    let report = validate_hpc_execution_resolver_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_execution_resolver(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcExecutionResolverReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_execution_resolver(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&absolute_output, render_hpc_execution_resolver_tsv(&report.rows))
        .with_context(|| format!("write {}", absolute_output.display()))?;
    Ok(report)
}

pub(crate) fn validate_hpc_execution_resolver_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcExecutionResolverReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let observed = fs::read_to_string(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected = build_hpc_execution_resolver(repo_root, &absolute_manifest_path)?;
    let expected_tsv = render_hpc_execution_resolver_tsv(&expected.rows);
    if observed != expected_tsv {
        return Err(anyhow!(
            "HPC execution resolver `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-execution-resolver --output {}`",
            absolute_manifest_path.display(),
            path_relative_to_repo(repo_root, &absolute_manifest_path)
        ));
    }
    Ok(expected)
}

fn build_hpc_execution_resolver(
    repo_root: &Path,
    absolute_output: &Path,
) -> Result<LocalHpcExecutionResolverReport> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output,
        "HPC execution resolver output",
    )?;
    let selected_tools = collect_selected_tool_scope(repo_root)?;
    let execution_modes =
        load_tool_execution_mode_assignments(&repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH))?;
    let apptainer_by_tool = collect_apptainer_map_rows(repo_root)?
        .into_iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = selected_tools
        .iter()
        .map(|scope| {
            build_execution_resolver_row(repo_root, scope, &execution_modes, &apptainer_by_tool)
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_execution_resolver_contract(&rows)?;

    let mut resolution_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *resolution_counts.entry(row.resolution_kind.clone()).or_default() += 1;
    }

    Ok(LocalHpcExecutionResolverReport {
        schema_version: LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output),
        row_count: rows.len(),
        unclassified_tool_count: rows
            .iter()
            .filter(|row| row.execution_mode == UNCLASSIFIED_EXECUTION_MODE)
            .count(),
        resolution_counts,
        rows,
    })
}

fn collect_selected_tool_scope(repo_root: &Path) -> Result<Vec<SelectedToolScope>> {
    let mut scope_by_tool = BTreeMap::<String, SelectedToolScopeAccumulator>::new();
    for job in load_local_hpc_selected_jobs(repo_root)? {
        let scope = scope_by_tool.entry(job.tool_id.clone()).or_default();
        scope.domains.insert(job.domain);
        scope.selected_stage_ids.insert(job.stage_id);
        scope.selected_job_ids.insert(job.job_id_local);
        if let Some(result_id) = job.result_id {
            scope.selected_result_ids.insert(result_id);
        }
        if let Some(pipeline_id) = job.pipeline_id {
            scope.selected_pipeline_ids.insert(pipeline_id);
        }
        if let Some(node_id) = job.node_id {
            scope.selected_node_ids.insert(node_id);
        }
    }

    let mut scopes = scope_by_tool
        .into_iter()
        .map(|(tool_id, scope)| SelectedToolScope {
            lookup_tool_id: canonical_lookup_tool_id(&tool_id),
            tool_id,
            domains: scope.domains.into_iter().collect(),
            selected_stage_ids: scope.selected_stage_ids.into_iter().collect(),
            selected_job_ids: scope.selected_job_ids.into_iter().collect(),
            selected_result_ids: scope.selected_result_ids.into_iter().collect(),
            selected_pipeline_ids: scope.selected_pipeline_ids.into_iter().collect(),
            selected_node_ids: scope.selected_node_ids.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    scopes.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(scopes)
}

fn build_execution_resolver_row(
    repo_root: &Path,
    scope: &SelectedToolScope,
    execution_modes: &BTreeMap<String, ToolExecutionModeAssignment>,
    apptainer_by_tool: &BTreeMap<String, ApptainerMapRow>,
) -> Result<LocalHpcExecutionResolverRow> {
    let assignment = execution_modes.get(scope.lookup_tool_id.as_str());
    let execution_mode = assignment
        .map(|value| value.execution_mode.clone())
        .unwrap_or_else(|| UNCLASSIFIED_EXECUTION_MODE.to_string());
    let execution_mode_summary = assignment
        .map(|value| value.summary.clone())
        .unwrap_or_else(|| UNCLASSIFIED_EXECUTION_MODE_SUMMARY.to_string());
    let apptainer_row = apptainer_by_tool.get(scope.lookup_tool_id.as_str());

    match observe_runtime_probe_resolution(repo_root, scope) {
        Ok(runtime_probe) => {
            let resolution = resolve_hpc_resolution(scope, &runtime_probe.signature, apptainer_row);
            Ok(LocalHpcExecutionResolverRow {
                tool_id: scope.tool_id.clone(),
                lookup_tool_id: scope.lookup_tool_id.clone(),
                execution_mode,
                execution_mode_summary,
                domains: scope.domains.clone(),
                selected_stage_ids: scope.selected_stage_ids.clone(),
                selected_job_count: scope.selected_job_ids.len(),
                selected_job_ids: scope.selected_job_ids.clone(),
                selected_result_ids: scope.selected_result_ids.clone(),
                selected_pipeline_ids: scope.selected_pipeline_ids.clone(),
                selected_node_ids: scope.selected_node_ids.clone(),
                install_kind: runtime_probe.signature.install_kind,
                resolution_kind: resolution.resolution_kind,
                resolution_target: resolution.resolution_target,
                command_entrypoint: runtime_probe.signature.command_entrypoint,
                runtime_probe_paths: runtime_probe.runtime_probe_paths,
                registry_paths: resolution.registry_paths,
                unavailable_reason: resolution.unavailable_reason,
            })
        }
        Err(err) => Ok(LocalHpcExecutionResolverRow {
            tool_id: scope.tool_id.clone(),
            lookup_tool_id: scope.lookup_tool_id.clone(),
            execution_mode,
            execution_mode_summary,
            domains: scope.domains.clone(),
            selected_stage_ids: scope.selected_stage_ids.clone(),
            selected_job_count: scope.selected_job_ids.len(),
            selected_job_ids: scope.selected_job_ids.clone(),
            selected_result_ids: scope.selected_result_ids.clone(),
            selected_pipeline_ids: scope.selected_pipeline_ids.clone(),
            selected_node_ids: scope.selected_node_ids.clone(),
            install_kind: "unknown".to_string(),
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            command_entrypoint: None,
            runtime_probe_paths: Vec::new(),
            registry_paths: Vec::new(),
            unavailable_reason: Some(err.to_string()),
        }),
    }
}

fn observe_runtime_probe_resolution(
    repo_root: &Path,
    scope: &SelectedToolScope,
) -> Result<ResolvedRuntimeProbeObservation> {
    let mut runtime_probe_paths = Vec::<String>::new();
    let mut resolved_signature = None::<RuntimeResolutionSignature>;

    for domain in &scope.domains {
        let loaded = load_runtime_probe_with_source(repo_root, domain, &scope.lookup_tool_id)
            .with_context(|| {
                format!(
                    "load governed runtime probe for selected tool `{}` via lookup tool `{}` in domain `{}`",
                    scope.tool_id, scope.lookup_tool_id, domain
                )
            })?;
        let signature = resolve_runtime_probe_signature(&scope.lookup_tool_id, &loaded.probe);
        let probe_path = path_relative_to_repo(repo_root, &loaded.path);
        if !runtime_probe_paths.iter().any(|existing| existing == &probe_path) {
            runtime_probe_paths.push(probe_path);
        }
        if let Some(previous) = &resolved_signature {
            if previous != &signature {
                return Err(anyhow!(
                    "selected tool `{}` resolves inconsistently across selected domains",
                    scope.tool_id
                ));
            }
        } else {
            resolved_signature = Some(signature);
        }
    }

    let signature = resolved_signature.ok_or_else(|| {
        anyhow!(
            "selected tool `{}` must resolve at least one governed runtime probe",
            scope.tool_id
        )
    })?;
    runtime_probe_paths.sort();

    Ok(ResolvedRuntimeProbeObservation { signature, runtime_probe_paths })
}

#[derive(Debug, Clone)]
struct HpcResolution {
    resolution_kind: String,
    resolution_target: String,
    registry_paths: Vec<String>,
    unavailable_reason: Option<String>,
}

fn resolve_hpc_resolution(
    scope: &SelectedToolScope,
    signature: &RuntimeResolutionSignature,
    apptainer_row: Option<&ApptainerMapRow>,
) -> HpcResolution {
    if signature.resolution_kind == RESOLUTION_KIND_HOST_BINARY {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_HOST_BINARY.to_string(),
            resolution_target: signature.resolution_target.clone(),
            registry_paths: Vec::new(),
            unavailable_reason: None,
        };
    }

    if let Some(apptainer_row) = apptainer_row {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
            resolution_target: apptainer_row.expected_sif_path.clone(),
            registry_paths: apptainer_row.registry_paths.clone(),
            unavailable_reason: None,
        };
    }

    if signature.resolution_kind == RESOLUTION_KIND_APPTAINER_IMAGE {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_APPTAINER_IMAGE.to_string(),
            resolution_target: signature.resolution_target.clone(),
            registry_paths: Vec::new(),
            unavailable_reason: None,
        };
    }

    if signature.resolution_kind == RESOLUTION_KIND_DOCKER_IMAGE {
        return HpcResolution {
            resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
            resolution_target: String::new(),
            registry_paths: Vec::new(),
            unavailable_reason: Some(format!(
                "selected tool `{}` is docker-backed in the governed runtime probe but lacks a governed Apptainer conversion",
                scope.tool_id
            )),
        };
    }

    HpcResolution {
        resolution_kind: RESOLUTION_KIND_UNAVAILABLE.to_string(),
        resolution_target: String::new(),
        registry_paths: Vec::new(),
        unavailable_reason: signature.unavailable_reason.clone(),
    }
}

fn canonical_lookup_tool_id(tool_id: &str) -> String {
    match tool_id {
        "bijux-dna" => "bijux_dna".to_string(),
        _ => tool_id.to_string(),
    }
}

fn render_hpc_execution_resolver_tsv(rows: &[LocalHpcExecutionResolverRow]) -> String {
    let mut body = String::from(
        "tool_id\tlookup_tool_id\texecution_mode\texecution_mode_summary\tdomains\tselected_stage_ids\tselected_job_count\tselected_job_ids\tselected_result_ids\tselected_pipeline_ids\tselected_node_ids\tinstall_kind\tresolution_kind\tresolution_target\tcommand_entrypoint\truntime_probe_paths\tregistry_paths\tunavailable_reason\n",
    );
    for row in rows {
        body.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv_field(&row.tool_id),
            sanitize_tsv_field(&row.lookup_tool_id),
            sanitize_tsv_field(&row.execution_mode),
            sanitize_tsv_field(&row.execution_mode_summary),
            sanitize_tsv_field(&row.domains.join(",")),
            sanitize_tsv_field(&row.selected_stage_ids.join(",")),
            row.selected_job_count,
            sanitize_tsv_field(&row.selected_job_ids.join(",")),
            sanitize_tsv_field(&row.selected_result_ids.join(",")),
            sanitize_tsv_field(&row.selected_pipeline_ids.join(",")),
            sanitize_tsv_field(&row.selected_node_ids.join(",")),
            sanitize_tsv_field(&row.install_kind),
            sanitize_tsv_field(&row.resolution_kind),
            sanitize_tsv_field(&row.resolution_target),
            sanitize_tsv_field(row.command_entrypoint.as_deref().unwrap_or("")),
            sanitize_tsv_field(&row.runtime_probe_paths.join(",")),
            sanitize_tsv_field(&row.registry_paths.join(",")),
            sanitize_tsv_field(row.unavailable_reason.as_deref().unwrap_or("")),
        ));
    }
    body
}

fn sanitize_tsv_field(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn ensure_execution_resolver_contract(rows: &[LocalHpcExecutionResolverRow]) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!("HPC execution resolver must cover at least one selected tool"));
    }

    let observed_tool_ids = rows.iter().map(|row| row.tool_id.as_str()).collect::<Vec<_>>();
    let sorted_tool_ids = {
        let mut sorted = observed_tool_ids.clone();
        sorted.sort_unstable();
        sorted
    };
    if observed_tool_ids != sorted_tool_ids {
        return Err(anyhow!("HPC execution resolver rows must remain sorted lexically by tool_id"));
    }

    let unique_tool_ids = observed_tool_ids.iter().copied().collect::<BTreeSet<_>>();
    if unique_tool_ids.len() != rows.len() {
        return Err(anyhow!("HPC execution resolver rows must keep tool_id values unique"));
    }

    for row in rows {
        if row.selected_job_count == 0 {
            return Err(anyhow!(
                "HPC execution resolver row `{}` must cover at least one selected job",
                row.tool_id
            ));
        }
        if row.selected_job_count != row.selected_job_ids.len() {
            return Err(anyhow!(
                "HPC execution resolver row `{}` selected_job_count must match selected_job_ids length",
                row.tool_id
            ));
        }
        if row.execution_mode.trim().is_empty() || row.execution_mode_summary.trim().is_empty() {
            return Err(anyhow!(
                "HPC execution resolver row `{}` must keep execution mode metadata populated",
                row.tool_id
            ));
        }
        if row.domains.is_empty() || row.selected_stage_ids.is_empty() {
            return Err(anyhow!(
                "HPC execution resolver row `{}` must preserve selected domain and stage scope",
                row.tool_id
            ));
        }
        match row.resolution_kind.as_str() {
            RESOLUTION_KIND_HOST_BINARY | RESOLUTION_KIND_APPTAINER_IMAGE => {
                if row.resolution_target.trim().is_empty() {
                    return Err(anyhow!(
                        "HPC execution resolver row `{}` must populate resolution_target for `{}`",
                        row.tool_id,
                        row.resolution_kind
                    ));
                }
                if row.unavailable_reason.is_some() {
                    return Err(anyhow!(
                        "HPC execution resolver row `{}` must not carry unavailable_reason when resolution_kind is `{}`",
                        row.tool_id,
                        row.resolution_kind
                    ));
                }
            }
            RESOLUTION_KIND_UNAVAILABLE => {
                if row
                    .unavailable_reason
                    .as_deref()
                    .map(str::trim)
                    .is_none_or(|value| value.is_empty())
                {
                    return Err(anyhow!(
                        "HPC execution resolver row `{}` must explain unavailable resolution",
                        row.tool_id
                    ));
                }
                if !row.resolution_target.is_empty() {
                    return Err(anyhow!(
                        "HPC execution resolver row `{}` must leave resolution_target empty when unavailable",
                        row.tool_id
                    ));
                }
            }
            other => {
                return Err(anyhow!(
                    "HPC execution resolver row `{}` uses unsupported resolution_kind `{other}`",
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}
