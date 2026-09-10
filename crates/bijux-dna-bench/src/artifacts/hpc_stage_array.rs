//! Owner: deterministic stage benchmark Slurm array serialization.
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalHpcStageBenchmarkArrayReport;
use std::path::Path;
const STAGE_BENCHMARK_ARRAY_JOB_NAME: &str = "bijux-stage-benchmark-array";
pub(crate) fn render_stage_benchmark_array_script(
    repo_root: &Path,
    report: &BenchmarkLocalHpcStageBenchmarkArrayReport,
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
        job_name = quote_hpc_shell_value(STAGE_BENCHMARK_ARRAY_JOB_NAME),
        repo_root = quote_hpc_shell_value(&repo_root_absolute),
        array_spec = report.array_spec,
        cpus_per_task = report.resource_envelope.cpus_per_task,
        memory_mb = report.resource_envelope.memory_mb,
        time_limit = report.resource_envelope.time_limit,
        wrapper_stdout = quote_hpc_shell_value(&wrapper_stdout_path),
        wrapper_stderr = quote_hpc_shell_value(&wrapper_stderr_path),
        manifest_path = quote_hpc_shell_value(&report.manifest_path),
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
            job_id_local = quote_hpc_shell_value(&row.job_id_local),
            result_id = quote_hpc_shell_value(&row.result_id),
            domain = quote_hpc_shell_value(&row.domain),
            stage_id = quote_hpc_shell_value(&row.stage_id),
            tool_id = quote_hpc_shell_value(&row.tool_id),
            corpus_id = quote_hpc_shell_value(&row.corpus_id),
            asset_profile_id = quote_hpc_shell_value(&row.asset_profile_id),
            execution_mode = quote_hpc_shell_value(&row.execution_mode),
            execution_mode_summary = quote_hpc_shell_value(&row.execution_mode_summary),
            resolution_kind = quote_hpc_shell_value(&row.resolution_kind),
            resolution_target = quote_hpc_shell_value(&row.resolution_target),
            unavailable_reason =
                quote_hpc_shell_value(row.unavailable_reason.as_deref().unwrap_or("")),
            output_root = quote_hpc_shell_value(&row.output_root),
            log_root = quote_hpc_shell_value(&row.log_root),
            stdout_path = quote_hpc_shell_value(&row.stdout_path),
            stderr_path = quote_hpc_shell_value(&row.stderr_path),
            execution_command = quote_hpc_shell_value(&row.benchmark_result_command),
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

/// Quote a shell assignment or Slurm path, retaining scheduler percent placeholders.
#[must_use]
pub fn quote_hpc_shell_value(value: &str) -> String {
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+' | '%')
    }) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
