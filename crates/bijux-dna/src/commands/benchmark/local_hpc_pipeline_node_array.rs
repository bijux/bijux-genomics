use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_job_execution::{
    render_shell_command, rendered_essential_pipeline_node_execution_argv,
};
use super::local_hpc_array_support::{
    manifest_path_for_script, shell_quote, time_limit_to_seconds, LocalHpcArrayResources,
};
use super::local_hpc_input_discovery::path_relative_to_repo;
use super::local_hpc_scratch_layout::{collect_hpc_scratch_layout, LocalHpcScratchLayoutJob};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
    LocalPipelineDagValidationNodeReport, LocalPipelineDagValidationReport,
};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use super::readiness::essential_pipeline_corpus_assets::ESSENTIAL_PIPELINE_IDS;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_pipeline_node_array.v1";
pub(crate) const DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH: &str =
    "runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch";
const PIPELINE_NODE_ARRAY_JOB_NAME: &str = "bijux-pipeline-node-array";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcPipelineNodeArrayExpectedOutput {
    pub(crate) output_id: String,
    pub(crate) output_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcPipelineNodeArrayDependency {
    pub(crate) node_id: String,
    pub(crate) job_id_local: String,
    pub(crate) array_index: usize,
    pub(crate) upstream_result_ids: Vec<String>,
    pub(crate) expected_outputs: Vec<LocalHpcPipelineNodeArrayExpectedOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcPipelineNodeArrayRow {
    pub(crate) array_index: usize,
    pub(crate) job_id_local: String,
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) dependency_job_ids: Vec<String>,
    pub(crate) upstream_result_ids: Vec<String>,
    pub(crate) dependencies: Vec<LocalHpcPipelineNodeArrayDependency>,
    pub(crate) expected_outputs: Vec<LocalHpcPipelineNodeArrayExpectedOutput>,
    pub(crate) script_path: String,
    pub(crate) pipeline_node_argv: Vec<String>,
    pub(crate) pipeline_node_command: String,
    pub(crate) scratch_root: String,
    pub(crate) input_links_root: String,
    pub(crate) output_root: String,
    pub(crate) log_root: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) resources: LocalHpcArrayResources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcPipelineNodeArrayReport {
    pub(crate) schema_version: &'static str,
    pub(crate) script_path: String,
    pub(crate) manifest_path: String,
    pub(crate) wrapper_log_root: String,
    pub(crate) array_spec: String,
    pub(crate) pipeline_count: usize,
    pub(crate) pipeline_job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) resource_envelope: LocalHpcArrayResources,
    pub(crate) rows: Vec<LocalHpcPipelineNodeArrayRow>,
}

struct BuiltLocalHpcPipelineNodeArray {
    report: LocalHpcPipelineNodeArrayReport,
    script_body: String,
    manifest_path: PathBuf,
}

pub(crate) fn run_render_hpc_pipeline_node_array(
    args: &parse::BenchLocalRenderHpcPipelineNodeArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH))
    });
    let report = render_hpc_pipeline_node_array(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_pipeline_node_array(
    args: &parse::BenchLocalValidateHpcPipelineNodeArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let script_path = args.script.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH))
    });
    let report = validate_hpc_pipeline_node_array_path(&repo_root, &script_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_pipeline_node_array(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let built = build_hpc_pipeline_node_array(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = built.manifest_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&absolute_output, built.script_body)
        .with_context(|| format!("write {}", absolute_output.display()))?;
    bijux_dna_infra::atomic_write_json(&built.manifest_path, &built.report)?;
    Ok(built.report)
}

pub(crate) fn collect_hpc_pipeline_node_array(
    repo_root: &Path,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let script_path = benchmark_paths
        .resolve_repo_relative(Path::new(DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH));
    build_hpc_pipeline_node_array(repo_root, &script_path).map(|built| built.report)
}

pub(crate) fn validate_hpc_pipeline_node_array_path(
    repo_root: &Path,
    script_path: &Path,
) -> Result<LocalHpcPipelineNodeArrayReport> {
    let absolute_script_path = if script_path.is_absolute() {
        script_path.to_path_buf()
    } else {
        repo_root.join(script_path)
    };
    let built = build_hpc_pipeline_node_array(repo_root, &absolute_script_path)?;
    let observed_script = fs::read_to_string(&absolute_script_path)
        .with_context(|| format!("read {}", absolute_script_path.display()))?;
    if observed_script != built.script_body {
        return Err(anyhow!(
            "HPC pipeline node array script `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-pipeline-node-array --output {}`",
            absolute_script_path.display(),
            built.report.script_path
        ));
    }

    let observed_manifest = fs::read(&built.manifest_path)
        .with_context(|| format!("read {}", built.manifest_path.display()))?;
    let expected_manifest = serde_json::to_vec_pretty(&built.report)
        .context("serialize governed HPC pipeline node array manifest")?;
    if observed_manifest != expected_manifest {
        return Err(anyhow!(
            "HPC pipeline node array manifest `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-pipeline-node-array --output {}`",
            built.manifest_path.display(),
            built.report.script_path
        ));
    }

    Ok(built.report)
}

fn build_hpc_pipeline_node_array(
    repo_root: &Path,
    absolute_script_path: &Path,
) -> Result<BuiltLocalHpcPipelineNodeArray> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_script_path,
        "HPC pipeline node array output",
    )?;
    let manifest_path = manifest_path_for_script("HPC pipeline node array", absolute_script_path)?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &manifest_path,
        "HPC pipeline node array manifest output",
    )?;

    let scratch_layout = collect_hpc_scratch_layout(repo_root)?;
    let scratch_by_node = scratch_layout
        .jobs
        .into_iter()
        .filter_map(|job| {
            let pipeline_id = job.pipeline_id.clone()?;
            let node_id = job.node_id.clone()?;
            Some(((pipeline_id, node_id), job))
        })
        .collect::<BTreeMap<_, _>>();
    let selected_jobs = load_local_hpc_selected_jobs(repo_root)?;
    let selected_pipeline_jobs = selected_jobs
        .into_iter()
        .filter(|job| job.pipeline_id.is_some() && job.node_id.is_some() && job.result_id.is_none())
        .collect::<Vec<_>>();
    let selected_job_by_node = selected_pipeline_jobs
        .into_iter()
        .map(|job| {
            let pipeline_id = job.pipeline_id.clone().expect("pipeline_id filtered above");
            let node_id = job.node_id.clone().expect("node_id filtered above");
            ((pipeline_id, node_id), job)
        })
        .collect::<BTreeMap<_, _>>();
    let pipeline_reports = load_pipeline_reports(repo_root)?;

    let mut rows = Vec::new();
    let mut row_index_by_job_id = BTreeMap::<String, usize>::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let pipeline = pipeline_reports.get(*pipeline_id).ok_or_else(|| {
            anyhow!("HPC pipeline node array is missing DAG validation report for `{pipeline_id}`")
        })?;
        let nodes_by_id = pipeline
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        let output_producers = output_producer_map(&pipeline.nodes)?;

        for node_id in &pipeline.topological_order {
            let node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing node `{node_id}` in pipeline `{pipeline_id}`"
                )
            })?;
            let key = ((*pipeline_id).to_string(), node.node_id.clone());
            let job = selected_job_by_node.get(&key).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing selected job coverage for `{pipeline_id}` / `{}`",
                    node.node_id
                )
            })?;
            let scratch = scratch_by_node.get(&key).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing scratch layout for `{pipeline_id}` / `{}`",
                    node.node_id
                )
            })?;
            let row = build_pipeline_node_array_row(
                rows.len(),
                pipeline,
                node,
                job,
                scratch,
                &nodes_by_id,
                &output_producers,
                &selected_job_by_node,
                &row_index_by_job_id,
            )?;
            row_index_by_job_id.insert(row.job_id_local.clone(), row.array_index);
            rows.push(row);
        }
    }

    if rows.len() != selected_job_by_node.len() {
        return Err(anyhow!(
            "HPC pipeline node array expected {} selected essential pipeline jobs but rendered {} rows",
            selected_job_by_node.len(),
            rows.len()
        ));
    }
    ensure_pipeline_node_array_contract(&rows)?;

    let script_parent = absolute_script_path.parent().ok_or_else(|| {
        anyhow!("pipeline node array output `{}` has no parent", absolute_script_path.display())
    })?;
    let file_stem =
        absolute_script_path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
            anyhow!(
                "pipeline node array output `{}` has no valid file stem",
                absolute_script_path.display()
            )
        })?;
    let wrapper_log_root = script_parent.join("logs").join(file_stem);
    let resource_envelope = compute_resource_envelope(&rows)?;
    let dependency_count = rows.iter().map(|row| row.dependencies.len()).sum::<usize>();
    let report = LocalHpcPipelineNodeArrayReport {
        schema_version: LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_VERSION,
        script_path: path_relative_to_repo(repo_root, absolute_script_path),
        manifest_path: path_relative_to_repo(repo_root, &manifest_path),
        wrapper_log_root: path_relative_to_repo(repo_root, &wrapper_log_root),
        array_spec: format!("0-{}", rows.len() - 1),
        pipeline_count: pipeline_reports.len(),
        pipeline_job_count: rows.len(),
        dependency_count,
        resource_envelope,
        rows,
    };
    let script_body = render_pipeline_node_array_script(repo_root, &report)?;
    Ok(BuiltLocalHpcPipelineNodeArray { report, script_body, manifest_path })
}

fn load_pipeline_reports(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalPipelineDagValidationReport>> {
    let mut reports = BTreeMap::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path = benchmark_local_pipeline_config_path(repo_root, pipeline_id);
        let report_path = repo_root
            .join("benchmarks/readiness/local-ready/pipeline-dag")
            .join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        reports.insert((*pipeline_id).to_string(), report);
    }
    Ok(reports)
}

fn output_producer_map(
    nodes: &[LocalPipelineDagValidationNodeReport],
) -> Result<BTreeMap<String, String>> {
    let mut producers = BTreeMap::new();
    for node in nodes {
        for output_id in &node.outputs {
            if let Some(previous) = producers.insert(output_id.clone(), node.node_id.clone()) {
                return Err(anyhow!(
                    "HPC pipeline node array found duplicate producer ownership for output `{output_id}`: `{previous}` and `{}`",
                    node.node_id
                ));
            }
        }
    }
    Ok(producers)
}

#[allow(clippy::too_many_arguments)]
fn build_pipeline_node_array_row(
    array_index: usize,
    pipeline: &LocalPipelineDagValidationReport,
    node: &LocalPipelineDagValidationNodeReport,
    job: &super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob,
    scratch: &LocalHpcScratchLayoutJob,
    nodes_by_id: &BTreeMap<String, &LocalPipelineDagValidationNodeReport>,
    output_producers: &BTreeMap<String, String>,
    selected_job_by_node: &BTreeMap<
        (String, String),
        super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob,
    >,
    row_index_by_job_id: &BTreeMap<String, usize>,
) -> Result<LocalHpcPipelineNodeArrayRow> {
    let pipeline_id = job.pipeline_id.clone().ok_or_else(|| {
        anyhow!(
            "HPC pipeline node array selected job `{}` is missing pipeline_id",
            job.job_id_local
        )
    })?;
    let node_id = job.node_id.clone().ok_or_else(|| {
        anyhow!("HPC pipeline node array selected job `{}` is missing node_id", job.job_id_local)
    })?;
    if pipeline_id != pipeline.pipeline_id || node_id != node.node_id {
        return Err(anyhow!(
            "HPC pipeline node array received mismatched job coverage for `{pipeline_id}` / `{node_id}`"
        ));
    }
    let expected_outputs = expected_outputs_for_job(node, job)?;
    let upstream_result_ids = node.upstream_inputs.clone();
    let dependency_job_ids = node
        .depends_on
        .iter()
        .map(|dependency_node_id| format!("pipeline:{pipeline_id}:{dependency_node_id}"))
        .collect::<Vec<_>>();
    if job.dependencies != dependency_job_ids {
        return Err(anyhow!(
            "HPC pipeline node array found submit-manifest dependency drift for `{pipeline_id}` / `{node_id}`"
        ));
    }

    let upstream_result_ids_by_dependency =
        group_upstream_result_ids_by_dependency(node, output_producers)?;
    let mut dependencies = Vec::with_capacity(node.depends_on.len());
    for dependency_node_id in &node.depends_on {
        let dependency_node = nodes_by_id.get(dependency_node_id).ok_or_else(|| {
            anyhow!(
                "HPC pipeline node array is missing dependency node `{dependency_node_id}` in pipeline `{pipeline_id}`"
            )
        })?;
        let dependency_job = selected_job_by_node
            .get(&(pipeline_id.clone(), dependency_node_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing selected dependency job `{pipeline_id}` / `{dependency_node_id}`"
                )
            })?;
        let dependency_job_id = dependency_job.job_id_local.clone();
        let dependency_array_index = row_index_by_job_id.get(&dependency_job_id).copied().ok_or_else(
            || {
                anyhow!(
                    "HPC pipeline node array dependency `{dependency_job_id}` for `{pipeline_id}` / `{node_id}` requires manual reordering"
                )
            },
        )?;
        let dependency_outputs = expected_outputs_for_job(dependency_node, dependency_job)?;
        let dependency_output_by_id = dependency_outputs
            .into_iter()
            .map(|output| (output.output_id.clone(), output))
            .collect::<BTreeMap<_, _>>();
        let upstream_result_ids =
            upstream_result_ids_by_dependency.get(dependency_node_id).cloned().unwrap_or_default();
        let expected_outputs = if upstream_result_ids.is_empty() {
            dependency_output_by_id.into_values().collect::<Vec<_>>()
        } else {
            upstream_result_ids
                .iter()
                .map(|result_id| {
                    dependency_output_by_id.get(result_id).cloned().ok_or_else(|| {
                        anyhow!(
                            "HPC pipeline node array dependency `{dependency_job_id}` is missing output `{result_id}` required by `{pipeline_id}` / `{node_id}`"
                        )
                    })
                })
                .collect::<Result<Vec<_>>>()?
        };
        dependencies.push(LocalHpcPipelineNodeArrayDependency {
            node_id: dependency_node_id.clone(),
            job_id_local: dependency_job_id,
            array_index: dependency_array_index,
            upstream_result_ids,
            expected_outputs,
        });
    }

    let pipeline_node_argv =
        rendered_essential_pipeline_node_execution_argv(&pipeline_id, &node_id);
    let pipeline_node_command = render_shell_command(&pipeline_node_argv);
    Ok(LocalHpcPipelineNodeArrayRow {
        array_index,
        job_id_local: job.job_id_local.clone(),
        pipeline_id,
        node_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        dependency_job_ids,
        upstream_result_ids,
        dependencies,
        expected_outputs,
        script_path: job.script_path.clone(),
        pipeline_node_argv,
        pipeline_node_command,
        scratch_root: scratch.scratch_root.clone(),
        input_links_root: scratch.input_links_root.clone(),
        output_root: scratch.output_root.clone(),
        log_root: scratch.log_root.clone(),
        stdout_path: scratch.stdout_path.clone(),
        stderr_path: scratch.stderr_path.clone(),
        resources: LocalHpcArrayResources {
            cpus_per_task: scratch.resources.cpus_per_task,
            memory_mb: scratch.resources.memory_mb,
            time_limit: scratch.resources.time_limit.clone(),
            scratch_gb: scratch.resources.scratch_gb,
        },
    })
}

fn expected_outputs_for_job(
    node: &LocalPipelineDagValidationNodeReport,
    job: &super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob,
) -> Result<Vec<LocalHpcPipelineNodeArrayExpectedOutput>> {
    if node.outputs.len() != job.outputs.len() {
        return Err(anyhow!(
            "HPC pipeline node array found {} declared outputs in the DAG but {} output paths in the submit manifest for `{}`",
            node.outputs.len(),
            job.outputs.len(),
            job.job_id_local
        ));
    }
    Ok(node
        .outputs
        .iter()
        .zip(job.outputs.iter())
        .map(|(output_id, output_path)| LocalHpcPipelineNodeArrayExpectedOutput {
            output_id: output_id.clone(),
            output_path: output_path.clone(),
        })
        .collect())
}

fn group_upstream_result_ids_by_dependency(
    node: &LocalPipelineDagValidationNodeReport,
    output_producers: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<String>>> {
    let declared_dependencies = node.depends_on.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut upstream_result_ids_by_dependency = BTreeMap::<String, Vec<String>>::new();
    for upstream_result_id in &node.upstream_inputs {
        let producer_node_id = output_producers.get(upstream_result_id).ok_or_else(|| {
            anyhow!(
                "HPC pipeline node array cannot resolve producer for upstream result `{upstream_result_id}` in node `{}`",
                node.node_id
            )
        })?;
        if !declared_dependencies.contains(producer_node_id.as_str()) {
            return Err(anyhow!(
                "HPC pipeline node array found undeclared producer `{producer_node_id}` for upstream result `{upstream_result_id}` in node `{}`",
                node.node_id
            ));
        }
        upstream_result_ids_by_dependency
            .entry(producer_node_id.clone())
            .or_default()
            .push(upstream_result_id.clone());
    }
    Ok(upstream_result_ids_by_dependency)
}

fn ensure_pipeline_node_array_contract(rows: &[LocalHpcPipelineNodeArrayRow]) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!(
            "HPC pipeline node array must cover at least one selected essential pipeline node"
        ));
    }
    if rows.iter().enumerate().any(|(index, row)| row.array_index != index) {
        return Err(anyhow!("HPC pipeline node array indexes must stay contiguous and zero-based"));
    }
    let unique_job_ids = rows.iter().map(|row| row.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if unique_job_ids.len() != rows.len() {
        return Err(anyhow!(
            "HPC pipeline node array must keep one unique local job id per array row"
        ));
    }
    let unique_pipeline_nodes = rows
        .iter()
        .map(|row| format!("{}:{}", row.pipeline_id, row.node_id))
        .collect::<BTreeSet<_>>();
    if unique_pipeline_nodes.len() != rows.len() {
        return Err(anyhow!(
            "HPC pipeline node array must keep one unique pipeline node per array row"
        ));
    }
    for row in rows {
        if row.expected_outputs.is_empty() {
            return Err(anyhow!(
                "HPC pipeline node array row `{}` must keep expected outputs",
                row.job_id_local
            ));
        }
        if row.pipeline_node_argv.is_empty() || row.pipeline_node_command.is_empty() {
            return Err(anyhow!(
                "HPC pipeline node array row `{}` must keep an execution command",
                row.job_id_local
            ));
        }
        if row.stdout_path.is_empty() || row.stderr_path.is_empty() {
            return Err(anyhow!(
                "HPC pipeline node array row `{}` must keep explicit stdout and stderr paths",
                row.job_id_local
            ));
        }
        if row.dependency_job_ids.len() != row.dependencies.len() {
            return Err(anyhow!(
                "HPC pipeline node array row `{}` must keep one dependency entry per dependency job id",
                row.job_id_local
            ));
        }
        if row
            .dependencies
            .iter()
            .enumerate()
            .any(|(index, dependency)| dependency.job_id_local != row.dependency_job_ids[index])
        {
            return Err(anyhow!(
                "HPC pipeline node array row `{}` must keep dependency entries aligned with dependency job ids",
                row.job_id_local
            ));
        }
        for dependency in &row.dependencies {
            if dependency.array_index >= row.array_index {
                return Err(anyhow!(
                    "HPC pipeline node array dependency `{}` must appear before dependent row `{}`",
                    dependency.job_id_local,
                    row.job_id_local
                ));
            }
            if dependency.expected_outputs.is_empty() {
                return Err(anyhow!(
                    "HPC pipeline node array dependency `{}` must keep expected outputs",
                    dependency.job_id_local
                ));
            }
        }
    }
    Ok(())
}

fn compute_resource_envelope(
    rows: &[LocalHpcPipelineNodeArrayRow],
) -> Result<LocalHpcArrayResources> {
    let max_time_limit = rows
        .iter()
        .max_by_key(|row| {
            time_limit_to_seconds("HPC pipeline node array", &row.resources.time_limit).unwrap_or(0)
        })
        .map(|row| row.resources.time_limit.clone())
        .ok_or_else(|| {
            anyhow!("HPC pipeline node array cannot compute an empty resource envelope")
        })?;
    Ok(LocalHpcArrayResources {
        cpus_per_task: rows.iter().map(|row| row.resources.cpus_per_task).max().unwrap_or(1),
        memory_mb: rows.iter().map(|row| row.resources.memory_mb).max().unwrap_or(1024),
        time_limit: max_time_limit,
        scratch_gb: rows.iter().map(|row| row.resources.scratch_gb).max().unwrap_or(1),
    })
}

fn render_pipeline_node_array_script(
    repo_root: &Path,
    report: &LocalHpcPipelineNodeArrayReport,
) -> Result<String> {
    let repo_root_absolute = repo_root.to_string_lossy().replace('\\', "/");
    let wrapper_stdout_path = format!("{}/%A_%a.wrapper.stdout.log", report.wrapper_log_root);
    let wrapper_stderr_path = format!("{}/%A_%a.wrapper.stderr.log", report.wrapper_log_root);
    let mut rendered = format!(
        "#!/usr/bin/env bash\n\
set -euo pipefail\n\
\n\
#SBATCH --job-name={job_name}\n\
#SBATCH --chdir={repo_root}\n\
#SBATCH --array={array_spec}\n\
#SBATCH --cpus-per-task={cpus_per_task}\n\
#SBATCH --mem={memory_mb}M\n\
#SBATCH --time={time_limit}\n\
#SBATCH --output={wrapper_stdout}\n\
#SBATCH --error={wrapper_stderr}\n\
\n\
# Governed HPC pipeline node array dry-run script.\n\
# manifest_path: {manifest_path}\n\
# pipeline_job_count: {pipeline_job_count}\n\
# dependency_count: {dependency_count}\n\
# resource_envelope_scratch_gb: {scratch_gb}\n\
\n\
REPO_ROOT={repo_root}\n\
MANIFEST_PATH={manifest_path}\n\
TASK_INDEX=\"${{SLURM_ARRAY_TASK_ID:-}}\"\n\
\n\
if [[ -z \"$TASK_INDEX\" ]]; then\n\
  echo \"SLURM_ARRAY_TASK_ID is required for {job_name}\" >&2\n\
  exit 64\n\
fi\n\
\n\
case \"$TASK_INDEX\" in\n",
        job_name = shell_quote(PIPELINE_NODE_ARRAY_JOB_NAME),
        repo_root = shell_quote(&repo_root_absolute),
        array_spec = report.array_spec,
        cpus_per_task = report.resource_envelope.cpus_per_task,
        memory_mb = report.resource_envelope.memory_mb,
        time_limit = report.resource_envelope.time_limit,
        wrapper_stdout = shell_quote(&wrapper_stdout_path),
        wrapper_stderr = shell_quote(&wrapper_stderr_path),
        manifest_path = shell_quote(&report.manifest_path),
        pipeline_job_count = report.pipeline_job_count,
        dependency_count = report.dependency_count,
        scratch_gb = report.resource_envelope.scratch_gb,
    );
    for row in &report.rows {
        rendered.push_str(&format!(
            "  {array_index})\n\
    JOB_ID_LOCAL={job_id_local}\n\
    PIPELINE_ID={pipeline_id}\n\
    NODE_ID={node_id}\n\
    DOMAIN={domain}\n\
    STAGE_ID={stage_id}\n\
    TOOL_ID={tool_id}\n\
    CORPUS_ID={corpus_id}\n\
    ASSET_PROFILE_ID={asset_profile_id}\n\
    DEPENDENCY_JOB_IDS={dependency_job_ids}\n\
    UPSTREAM_RESULT_IDS={upstream_result_ids}\n\
    OUTPUT_ROOT={output_root}\n\
    LOG_ROOT={log_root}\n\
    STDOUT_PATH={stdout_path}\n\
    STDERR_PATH={stderr_path}\n\
    EXECUTION_COMMAND={execution_command}\n\
    ;;\n",
            array_index = row.array_index,
            job_id_local = shell_quote(&row.job_id_local),
            pipeline_id = shell_quote(&row.pipeline_id),
            node_id = shell_quote(&row.node_id),
            domain = shell_quote(&row.domain),
            stage_id = shell_quote(&row.stage_id),
            tool_id = shell_quote(&row.tool_id),
            corpus_id = shell_quote(&row.corpus_id),
            asset_profile_id = shell_quote(&row.asset_profile_id),
            dependency_job_ids = shell_quote(&csv_or_none(&row.dependency_job_ids)),
            upstream_result_ids = shell_quote(&csv_or_none(&row.upstream_result_ids)),
            output_root = shell_quote(&row.output_root),
            log_root = shell_quote(&row.log_root),
            stdout_path = shell_quote(&row.stdout_path),
            stderr_path = shell_quote(&row.stderr_path),
            execution_command = shell_quote(&row.pipeline_node_command),
        ));
    }
    rendered.push_str(
        "  *)\n\
    echo \"unknown pipeline node array index: $TASK_INDEX ($MANIFEST_PATH)\" >&2\n\
    exit 64\n\
    ;;\n\
esac\n\
\n\
mkdir -p \"$(dirname \"$STDOUT_PATH\")\" \"$(dirname \"$STDERR_PATH\")\" \"$OUTPUT_ROOT\" \"$LOG_ROOT\"\n\
cd \"$REPO_ROOT\"\n\
sh -c \"$EXECUTION_COMMAND\" >\"$STDOUT_PATH\" 2>\"$STDERR_PATH\"\n",
    );
    Ok(rendered)
}

fn csv_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}
