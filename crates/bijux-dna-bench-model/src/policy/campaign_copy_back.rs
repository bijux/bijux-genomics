//! Pure campaign artifact projection and POSIX shell argument rendering.
use crate::{
    sidecar_path_for, CampaignDryRunReport, CopyBackEntry, CopyBackManifestReport,
    COPY_BACK_MANIFEST_SCHEMA_VERSION,
};
use std::path::{Path, PathBuf};

/// Project every planned job; the legacy suggested command illustrates the first job only.
#[must_use]
pub fn plan_campaign_copy_back(
    report: &CampaignDryRunReport,
    output_path: Option<&Path>,
) -> CopyBackManifestReport {
    let manifest_path = output_path.map_or_else(
        || {
            Path::new(&report.config_path).parent().map_or_else(
                || PathBuf::from("artifacts/slurm_copy_back_manifest.json"),
                |parent| parent.join("copy-back-manifest.json"),
            )
        },
        Path::to_path_buf,
    );

    let entries = report
        .planned_jobs
        .iter()
        .map(|job| CopyBackEntry {
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            scratch_dir: job.outputs.scratch_dir.clone(),
            log_path: job.outputs.log.clone(),
            out_path: job.outputs.out.clone(),
            err_path: job.outputs.err.clone(),
            results_path: job.outputs.results.clone(),
            results_sidecar_path: sidecar_path_for(Path::new(&job.outputs.results))
                .display()
                .to_string(),
            code_path: job.outputs.code.clone(),
            code_sidecar_path: sidecar_path_for(Path::new(&job.outputs.code)).display().to_string(),
            script_path: slurm_script_path(Path::new(&job.outputs.log)).display().to_string(),
        })
        .collect::<Vec<_>>();

    let suggested_copy_command = if let Some(first) = entries.first() {
        format!(
            "rsync -av {} {} {} {} {} {} {} <destination_dir>/",
            quote_slurm_argument(&first.log_path),
            quote_slurm_argument(&first.out_path),
            quote_slurm_argument(&first.err_path),
            quote_slurm_argument(&first.results_path),
            quote_slurm_argument(&first.results_sidecar_path),
            quote_slurm_argument(&first.code_path),
            quote_slurm_argument(&first.code_sidecar_path)
        )
    } else {
        "rsync -av <source> <destination_dir>/".to_string()
    };

    CopyBackManifestReport {
        schema_version: COPY_BACK_MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_path: manifest_path.display().to_string(),
        campaign_id: report.campaign_id.clone(),
        domain: report.domain.clone(),
        suggested_copy_command,
        entries,
    }
}

/// Quote one literal POSIX shell argument, including an empty argument.
#[must_use]
pub fn quote_slurm_argument(value: &str) -> String {
    if !value.is_empty()
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

/// Resolve the scheduler script adjacent to its declared log without filesystem effects.
#[must_use]
pub fn slurm_script_path(log_path: &Path) -> PathBuf {
    let mut script = log_path.to_path_buf();
    let new_name = match script.file_name().and_then(|s| s.to_str()) {
        Some(name) => format!("{name}.sbatch.sh"),
        None => "job.sbatch.sh".to_string(),
    };
    script.set_file_name(new_name);
    script
}
