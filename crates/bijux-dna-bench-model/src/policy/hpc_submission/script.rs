//! Deterministic campaign script projection; no filesystem or scheduler effects.
use crate::{
    quote_slurm_argument as shell_quote, CampaignDryRunReport, HpcBundleFoundationLocks, PlannedJob,
};
use anyhow::Result;

fn prepared_root_shell_block(
    report: &CampaignDryRunReport,
    locks: &HpcBundleFoundationLocks,
) -> String {
    let corpora_root = shell_quote(&report.layout.corpora_root);
    let databases_root = shell_quote(&report.layout.databases_root);
    let images_root = shell_quote(&report.layout.images_root);
    let corpora_lock = shell_quote(&locks.corpus_lock);
    let databases_lock = shell_quote(&locks.database_lock);
    let images_lock = shell_quote(&locks.image_lock);
    format!(
        "export BIJUX_CORPORA_ROOT={corpora_root}\n\
export BIJUX_DATABASES_ROOT={databases_root}\n\
export BIJUX_IMAGES_ROOT={images_root}\n\
export BIJUX_CORPORA_PREPARE_LOCK={corpora_lock}\n\
export BIJUX_DATABASES_PREPARE_LOCK={databases_lock}\n\
export BIJUX_IMAGES_PREPARE_LOCK={images_lock}\n\
for bijux_prepare_lock in \\\n\
  \"$BIJUX_CORPORA_PREPARE_LOCK\" \\\n\
  \"$BIJUX_DATABASES_PREPARE_LOCK\" \\\n\
  \"$BIJUX_IMAGES_PREPARE_LOCK\"; do\n\
  if [ ! -f \"$bijux_prepare_lock\" ]; then\n\
    echo \"missing prepared foundation lock: $bijux_prepare_lock\" >&2\n\
    exit 97\n\
  fi\n\
done\n"
    )
}

/// Render one attempt, with scheduler directives preceding every executable statement.
/// # Errors
/// Rejects control characters in scheduler headers and malformed dependency identities.
pub fn render_hpc_submission_script(
    report: &CampaignDryRunReport,
    job: &PlannedJob,
    script_path: &str,
    dependency_scheduler_ids: &[String],
    locks: &HpcBundleFoundationLocks,
) -> Result<String> {
    for (label, value) in [
        ("campaign", report.campaign_id.as_str()),
        ("domain", report.domain.as_str()),
        ("job", job.job_name.as_str()),
        ("stage", job.stage.as_str()),
        ("tool", job.tool.as_str()),
        ("sample", job.sample.as_str()),
        ("walltime", job.resources.walltime.as_str()),
        ("partition", report.resolved_slurm.partition.as_str()),
        ("qos", report.resolved_slurm.qos.as_str()),
        ("script path", script_path),
    ] {
        anyhow::ensure!(!value.chars().any(char::is_control), "control character in Slurm {label}");
    }
    for identity in dependency_scheduler_ids {
        anyhow::ensure!(
            !identity.is_empty()
                && identity
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'-' | b'_')),
            "invalid scheduler dependency identity"
        );
    }
    let dependency_line = if dependency_scheduler_ids.is_empty() {
        String::new()
    } else {
        format!("#SBATCH --dependency=afterok:{}\n", dependency_scheduler_ids.join(":"))
    };
    let array_line =
        job.array_task.map_or_else(String::new, |task| format!("#SBATCH --array={task}\n"));
    let prepared_root_block = prepared_root_shell_block(report, locks);

    Ok(format!(
        "#!/usr/bin/env bash\n#SBATCH --job-name={}\n#SBATCH --cpus-per-task={}\n#SBATCH --mem={}G\n#SBATCH --time={}\n#SBATCH --partition={}\n#SBATCH --qos={}\n{}{}\nset -euo pipefail\n\n# Campaign: {}\n# Domain: {}\n# Stage: {}\n# Tool: {}\n# Sample: {}\n# Script path: {}\n\nexport BIJUX_RUN_CONTEXT=hpc\nexport BIJUX_ARRAY_TASK=${{SLURM_ARRAY_TASK_ID:-{}}}\n{}\nexport BIJUX_SCRATCH_DIR={}\nexport BIJUX_SCRATCH_IN=$BIJUX_SCRATCH_DIR/in\nexport BIJUX_SCRATCH_OUT=$BIJUX_SCRATCH_DIR/out\nmkdir -p \"$BIJUX_SCRATCH_IN\" \"$BIJUX_SCRATCH_OUT\"\ncleanup() {{\n  rm -rf \"$BIJUX_SCRATCH_DIR\"\n}}\ntrap cleanup EXIT\n\nif [ -f {} ]; then\n  set -a\n  # shellcheck disable=SC1090\n  . {}\n  set +a\nfi\n\n# Each scheduler job executes exactly one attempt. Retry and supersession are\n# separate lifecycle transitions, never a blocking loop inside this script.\nprintf '%s array_task=%s scratch=%s\\n' {} \"$BIJUX_ARRAY_TASK\" \"$BIJUX_SCRATCH_DIR\"\n",
        shell_quote(job.job_name.as_str()),
        job.resources.cpus,
        job.resources.mem_gb,
        shell_quote(&job.resources.walltime),
        shell_quote(&report.resolved_slurm.partition),
        shell_quote(&report.resolved_slurm.qos),
        dependency_line,
        array_line,
        report.campaign_id,
        report.domain,
        job.stage,
        job.tool,
        job.sample,
        script_path,
        job.array_task.unwrap_or(0),
        prepared_root_block,
        shell_quote(&job.outputs.scratch_dir),
        shell_quote(&report.env_file_path),
        shell_quote(&report.env_file_path),
        shell_quote(&format!("execute stage {} tool {} sample {}", job.stage, job.tool, job.sample))
    ))
}
