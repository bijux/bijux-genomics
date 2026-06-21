use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_job_execution::{
    render_shell_command, rendered_benchmark_result_execution_argv,
};
use super::local_hpc_array_support::{
    manifest_path_for_script, shell_quote, time_limit_to_seconds, LocalHpcArrayResources,
};
use super::local_hpc_execution_resolver::{
    collect_hpc_execution_resolver_report, LocalHpcExecutionResolverRow,
};
use super::local_hpc_input_discovery::path_relative_to_repo;
use super::local_hpc_scratch_layout::{collect_hpc_scratch_layout, LocalHpcScratchLayoutJob};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_stage_benchmark_array.v1";
pub(crate) const DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH: &str =
    "runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch";
const STAGE_BENCHMARK_ARRAY_JOB_NAME: &str = "bijux-stage-benchmark-array";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcStageBenchmarkArrayRow {
    pub(crate) array_index: usize,
    pub(crate) job_id_local: String,
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) execution_mode: String,
    pub(crate) execution_mode_summary: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) unavailable_reason: Option<String>,
    pub(crate) script_path: String,
    pub(crate) benchmark_result_argv: Vec<String>,
    pub(crate) benchmark_result_command: String,
    pub(crate) scratch_root: String,
    pub(crate) input_links_root: String,
    pub(crate) output_root: String,
    pub(crate) log_root: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) output_paths: Vec<String>,
    pub(crate) resources: LocalHpcArrayResources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcStageBenchmarkArrayReport {
    pub(crate) schema_version: &'static str,
    pub(crate) script_path: String,
    pub(crate) manifest_path: String,
    pub(crate) wrapper_log_root: String,
    pub(crate) array_spec: String,
    pub(crate) benchmark_job_count: usize,
    pub(crate) resource_envelope: LocalHpcArrayResources,
    pub(crate) rows: Vec<LocalHpcStageBenchmarkArrayRow>,
}

struct BuiltLocalHpcStageBenchmarkArray {
    report: LocalHpcStageBenchmarkArrayReport,
    script_body: String,
    manifest_path: PathBuf,
}

pub(crate) fn run_render_hpc_stage_benchmark_array(
    args: &parse::BenchLocalRenderHpcStageBenchmarkArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH))
    });
    let report = render_hpc_stage_benchmark_array(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_stage_benchmark_array(
    args: &parse::BenchLocalValidateHpcStageBenchmarkArrayArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let script_path = args.script.clone().unwrap_or_else(|| {
        benchmark_paths
            .resolve_repo_relative(Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH))
    });
    let report = validate_hpc_stage_benchmark_array_path(&repo_root, &script_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.script_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_stage_benchmark_array(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let built = build_hpc_stage_benchmark_array(repo_root, &absolute_output)?;
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

pub(crate) fn collect_hpc_stage_benchmark_array(
    repo_root: &Path,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let script_path = benchmark_paths
        .resolve_repo_relative(Path::new(DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH));
    build_hpc_stage_benchmark_array(repo_root, &script_path).map(|built| built.report)
}

pub(crate) fn validate_hpc_stage_benchmark_array_path(
    repo_root: &Path,
    script_path: &Path,
) -> Result<LocalHpcStageBenchmarkArrayReport> {
    let absolute_script_path = if script_path.is_absolute() {
        script_path.to_path_buf()
    } else {
        repo_root.join(script_path)
    };
    let built = build_hpc_stage_benchmark_array(repo_root, &absolute_script_path)?;
    let observed_script = fs::read_to_string(&absolute_script_path)
        .with_context(|| format!("read {}", absolute_script_path.display()))?;
    if observed_script != built.script_body {
        return Err(anyhow!(
            "HPC stage benchmark array script `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-stage-benchmark-array --output {}`",
            absolute_script_path.display(),
            built.report.script_path
        ));
    }

    let observed_manifest = fs::read(&built.manifest_path)
        .with_context(|| format!("read {}", built.manifest_path.display()))?;
    let expected_manifest = serde_json::to_vec_pretty(&built.report)
        .context("serialize governed HPC stage benchmark array manifest")?;
    if observed_manifest != expected_manifest {
        return Err(anyhow!(
            "HPC stage benchmark array manifest `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-stage-benchmark-array --output {}`",
            built.manifest_path.display(),
            built.report.script_path
        ));
    }

    Ok(built.report)
}

fn build_hpc_stage_benchmark_array(
    repo_root: &Path,
    absolute_script_path: &Path,
) -> Result<BuiltLocalHpcStageBenchmarkArray> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_script_path,
        "HPC stage benchmark array output",
    )?;
    let manifest_path =
        manifest_path_for_script("HPC stage benchmark array", absolute_script_path)?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &manifest_path,
        "HPC stage benchmark array manifest output",
    )?;

    let scratch_layout = collect_hpc_scratch_layout(repo_root)?;
    let scratch_by_result = scratch_layout
        .jobs
        .into_iter()
        .filter_map(|job| job.result_id.clone().map(|result_id| (result_id, job)))
        .collect::<BTreeMap<_, _>>();
    let execution_resolver = collect_hpc_execution_resolver_report(repo_root)?;
    let execution_by_tool = execution_resolver
        .rows
        .into_iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut benchmark_jobs = load_local_hpc_selected_jobs(repo_root)?
        .into_iter()
        .filter(|job| job.result_id.is_some() && job.pipeline_id.is_none() && job.node_id.is_none())
        .collect::<Vec<_>>();
    benchmark_jobs.sort_by(|left, right| left.result_id.cmp(&right.result_id));

    let rows = benchmark_jobs
        .iter()
        .enumerate()
        .map(|(array_index, job)| {
            build_stage_benchmark_array_row(
                array_index,
                job,
                &scratch_by_result,
                &execution_by_tool,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    ensure_stage_benchmark_array_contract(&rows)?;

    let script_parent = absolute_script_path.parent().ok_or_else(|| {
        anyhow!("stage benchmark array output `{}` has no parent", absolute_script_path.display())
    })?;
    let file_stem =
        absolute_script_path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
            anyhow!(
                "stage benchmark array output `{}` has no valid file stem",
                absolute_script_path.display()
            )
        })?;
    let wrapper_log_root = script_parent.join("logs").join(file_stem);
    let array_spec = format!("0-{}", rows.len() - 1);
    let resource_envelope = compute_resource_envelope(&rows)?;

    let report = LocalHpcStageBenchmarkArrayReport {
        schema_version: LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_VERSION,
        script_path: path_relative_to_repo(repo_root, absolute_script_path),
        manifest_path: path_relative_to_repo(repo_root, &manifest_path),
        wrapper_log_root: path_relative_to_repo(repo_root, &wrapper_log_root),
        array_spec,
        benchmark_job_count: rows.len(),
        resource_envelope,
        rows,
    };
    let script_body = render_stage_benchmark_array_script(repo_root, &report)?;
    Ok(BuiltLocalHpcStageBenchmarkArray { report, script_body, manifest_path })
}

fn build_stage_benchmark_array_row(
    array_index: usize,
    job: &super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob,
    scratch_by_result: &BTreeMap<String, LocalHpcScratchLayoutJob>,
    execution_by_tool: &BTreeMap<String, LocalHpcExecutionResolverRow>,
) -> Result<LocalHpcStageBenchmarkArrayRow> {
    let result_id = job.result_id.clone().ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array benchmark job `{}` is missing result_id",
            job.job_id_local
        )
    })?;
    let scratch = scratch_by_result.get(&result_id).ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array is missing scratch layout for benchmark result `{result_id}`"
        )
    })?;
    let execution = execution_by_tool.get(&job.tool_id).ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array is missing execution resolver coverage for tool `{}`",
            job.tool_id
        )
    })?;
    if !execution.selected_job_ids.contains(&job.job_id_local) {
        return Err(anyhow!(
            "HPC stage benchmark array tool `{}` does not cover selected job `{}` in the execution resolver",
            job.tool_id,
            job.job_id_local
        ));
    }
    if !execution.selected_result_ids.iter().any(|value| value == &result_id) {
        return Err(anyhow!(
            "HPC stage benchmark array tool `{}` does not cover result `{result_id}` in the execution resolver",
            job.tool_id
        ));
    }
    let benchmark_result_argv = rendered_benchmark_result_execution_argv(&result_id);
    let benchmark_result_command = render_shell_command(&benchmark_result_argv);
    Ok(LocalHpcStageBenchmarkArrayRow {
        array_index,
        job_id_local: job.job_id_local.clone(),
        result_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        execution_mode: execution.execution_mode.clone(),
        execution_mode_summary: execution.execution_mode_summary.clone(),
        resolution_kind: execution.resolution_kind.clone(),
        resolution_target: execution.resolution_target.clone(),
        unavailable_reason: execution.unavailable_reason.clone(),
        script_path: job.script_path.clone(),
        benchmark_result_argv,
        benchmark_result_command,
        scratch_root: scratch.scratch_root.clone(),
        input_links_root: scratch.input_links_root.clone(),
        output_root: scratch.output_root.clone(),
        log_root: scratch.log_root.clone(),
        stdout_path: scratch.stdout_path.clone(),
        stderr_path: scratch.stderr_path.clone(),
        output_paths: job.outputs.clone(),
        resources: LocalHpcArrayResources {
            cpus_per_task: scratch.resources.cpus_per_task,
            memory_mb: scratch.resources.memory_mb,
            time_limit: scratch.resources.time_limit.clone(),
            scratch_gb: scratch.resources.scratch_gb,
        },
    })
}

fn ensure_stage_benchmark_array_contract(rows: &[LocalHpcStageBenchmarkArrayRow]) -> Result<()> {
    if rows.is_empty() {
        return Err(anyhow!(
            "HPC stage benchmark array must cover at least one selected benchmark result"
        ));
    }
    if rows.iter().enumerate().any(|(index, row)| row.array_index != index) {
        return Err(anyhow!(
            "HPC stage benchmark array indexes must stay contiguous and zero-based"
        ));
    }
    let unique_job_ids = rows.iter().map(|row| row.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if unique_job_ids.len() != rows.len() {
        return Err(anyhow!(
            "HPC stage benchmark array must keep one unique local job id per array row"
        ));
    }
    let unique_result_ids = rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    if unique_result_ids.len() != rows.len() {
        return Err(anyhow!(
            "HPC stage benchmark array must keep one unique benchmark result id per array row"
        ));
    }
    for row in rows {
        if row.output_paths.is_empty() {
            return Err(anyhow!(
                "HPC stage benchmark array row `{}` must keep at least one declared output",
                row.result_id
            ));
        }
        if row.stdout_path.is_empty() || row.stderr_path.is_empty() {
            return Err(anyhow!(
                "HPC stage benchmark array row `{}` must keep explicit stdout and stderr paths",
                row.result_id
            ));
        }
        if row.benchmark_result_argv.is_empty() || row.benchmark_result_command.is_empty() {
            return Err(anyhow!(
                "HPC stage benchmark array row `{}` must keep an execution command",
                row.result_id
            ));
        }
        if row.execution_mode.is_empty()
            || row.execution_mode_summary.is_empty()
            || row.resolution_kind.is_empty()
        {
            return Err(anyhow!(
                "HPC stage benchmark array row `{}` must keep execution resolution metadata",
                row.result_id
            ));
        }
    }
    Ok(())
}

fn compute_resource_envelope(
    rows: &[LocalHpcStageBenchmarkArrayRow],
) -> Result<LocalHpcArrayResources> {
    let max_time_limit = rows
        .iter()
        .max_by_key(|row| {
            time_limit_to_seconds("HPC stage benchmark array", &row.resources.time_limit)
                .unwrap_or(0)
        })
        .map(|row| row.resources.time_limit.clone())
        .ok_or_else(|| {
            anyhow!("HPC stage benchmark array cannot compute an empty resource envelope")
        })?;
    Ok(LocalHpcArrayResources {
        cpus_per_task: rows.iter().map(|row| row.resources.cpus_per_task).max().unwrap_or(1),
        memory_mb: rows.iter().map(|row| row.resources.memory_mb).max().unwrap_or(1024),
        time_limit: max_time_limit,
        scratch_gb: rows.iter().map(|row| row.resources.scratch_gb).max().unwrap_or(1),
    })
}

fn render_stage_benchmark_array_script(
    repo_root: &Path,
    report: &LocalHpcStageBenchmarkArrayReport,
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
# Governed HPC stage benchmark array dry-run script.\n\
# manifest_path: {manifest_path}\n\
# benchmark_job_count: {benchmark_job_count}\n\
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
        job_name = shell_quote(STAGE_BENCHMARK_ARRAY_JOB_NAME),
        repo_root = shell_quote(&repo_root_absolute),
        array_spec = report.array_spec,
        cpus_per_task = report.resource_envelope.cpus_per_task,
        memory_mb = report.resource_envelope.memory_mb,
        time_limit = report.resource_envelope.time_limit,
        wrapper_stdout = shell_quote(&wrapper_stdout_path),
        wrapper_stderr = shell_quote(&wrapper_stderr_path),
        manifest_path = shell_quote(&report.manifest_path),
        benchmark_job_count = report.benchmark_job_count,
        scratch_gb = report.resource_envelope.scratch_gb,
    );
    for row in &report.rows {
        rendered.push_str(&format!(
            "  {array_index})\n\
    JOB_ID_LOCAL={job_id_local}\n\
    RESULT_ID={result_id}\n\
    DOMAIN={domain}\n\
    STAGE_ID={stage_id}\n\
    TOOL_ID={tool_id}\n\
    CORPUS_ID={corpus_id}\n\
    ASSET_PROFILE_ID={asset_profile_id}\n\
    EXECUTION_MODE={execution_mode}\n\
    EXECUTION_MODE_SUMMARY={execution_mode_summary}\n\
    RESOLUTION_KIND={resolution_kind}\n\
    RESOLUTION_TARGET={resolution_target}\n\
    UNAVAILABLE_REASON={unavailable_reason}\n\
    OUTPUT_ROOT={output_root}\n\
    LOG_ROOT={log_root}\n\
    STDOUT_PATH={stdout_path}\n\
    STDERR_PATH={stderr_path}\n\
    EXECUTION_COMMAND={execution_command}\n\
    ;;\n",
            array_index = row.array_index,
            job_id_local = shell_quote(&row.job_id_local),
            result_id = shell_quote(&row.result_id),
            domain = shell_quote(&row.domain),
            stage_id = shell_quote(&row.stage_id),
            tool_id = shell_quote(&row.tool_id),
            corpus_id = shell_quote(&row.corpus_id),
            asset_profile_id = shell_quote(&row.asset_profile_id),
            execution_mode = shell_quote(&row.execution_mode),
            execution_mode_summary = shell_quote(&row.execution_mode_summary),
            resolution_kind = shell_quote(&row.resolution_kind),
            resolution_target = shell_quote(&row.resolution_target),
            unavailable_reason = shell_quote(row.unavailable_reason.as_deref().unwrap_or("")),
            output_root = shell_quote(&row.output_root),
            log_root = shell_quote(&row.log_root),
            stdout_path = shell_quote(&row.stdout_path),
            stderr_path = shell_quote(&row.stderr_path),
            execution_command = shell_quote(&row.benchmark_result_command),
        ));
    }
    rendered.push_str(
        "  *)\n\
    echo \"unknown stage benchmark array index: $TASK_INDEX ($MANIFEST_PATH)\" >&2\n\
    exit 64\n\
    ;;\n\
esac\n\
\n\
mkdir -p \"$(dirname \"$STDOUT_PATH\")\" \"$(dirname \"$STDERR_PATH\")\" \"$OUTPUT_ROOT\" \"$LOG_ROOT\"\n\
cd \"$REPO_ROOT\"\n\
BIJUX_HPC_EXECUTION_MODE=\"$EXECUTION_MODE\" \\\n\
BIJUX_HPC_EXECUTION_MODE_SUMMARY=\"$EXECUTION_MODE_SUMMARY\" \\\n\
BIJUX_HPC_RESOLUTION_KIND=\"$RESOLUTION_KIND\" \\\n\
BIJUX_HPC_RESOLUTION_TARGET=\"$RESOLUTION_TARGET\" \\\n\
BIJUX_HPC_UNAVAILABLE_REASON=\"$UNAVAILABLE_REASON\" \\\n\
sh -c \"$EXECUTION_COMMAND\" >\"$STDOUT_PATH\" 2>\"$STDERR_PATH\"\n",
    );
    Ok(rendered)
}
