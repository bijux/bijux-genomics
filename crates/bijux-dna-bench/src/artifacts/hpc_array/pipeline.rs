//! Owner: deterministic Slurm serialization for admitted pipeline-node arrays.
use super::shell::quote_hpc_shell_value as shell_quote;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalHpcPipelineNodeArrayReport;
use std::path::Path;
const PIPELINE_NODE_ARRAY_JOB_NAME: &str = "bijux-pipeline-node-array";
/// Render the governed pipeline-node array shell artifact.
/// # Errors
/// Returns serialization errors.
pub(crate) fn render_hpc_pipeline_array_script(
    repo_root: &Path,
    report: &BenchmarkLocalHpcPipelineNodeArrayReport,
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
