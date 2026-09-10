//! Owner: ordered campaign submission with explicit scheduler and encrypted publication capabilities.
use super::collect_hpc_redaction_needles;
use crate::repo::hpc_campaign::submission::{publish_submission_script, write_operator_files};
use anyhow::Result;
use bijux_dna_bench_model::{
    plan_hpc_submission, render_hpc_submission_script, slurm_script_path as script_path_for,
    validate_hpc_submission_report, CampaignDryRunReport, HpcBundleFoundationLocks,
    HpcBundleSubmission, HpcSubmissionMode as SubmissionMode,
    HpcSubmissionSettings as SubmissionSettings, SlurmSubmissionReport, SubmittedJob,
    HPC_SLURM_SUBMISSION_SCHEMA_VERSION as SLURM_SUBMISSION_SCHEMA_VERSION,
};
use std::path::Path;
mod backend;
mod bundles;
pub use backend::HpcSubmissionBackend;
use bundles::SubmissionBundlePublisher;

/// Submit selected jobs in dependency order and return admitted receipts.
/// # Errors
/// Stops at the first preparation, scheduler or publication failure; prior external effects may remain.
pub fn submit_hpc_campaign(
    report: CampaignDryRunReport,
    settings: SubmissionSettings,
    locks: &HpcBundleFoundationLocks,
    submitted_at: String,
    backend: &impl HpcSubmissionBackend,
) -> Result<SlurmSubmissionReport> {
    anyhow::ensure!(
        !matches!(settings.mode, SubmissionMode::Real) || !report.security.encrypt_operator_outputs,
        "real Slurm submission cannot encrypt active scheduler output paths; collect terminal outputs before encryption"
    );
    let plan = plan_hpc_submission(&report.planned_jobs, &settings.subset)?;
    let redaction_needles = collect_hpc_redaction_needles(
        Path::new(&report.env_file_path),
        &report.security.redacted_env_keys,
    )?;

    let mut scheduler_ids = Vec::new();
    let mut jobs_out = Vec::new();

    for (index, selected_job) in plan.jobs().iter().enumerate() {
        let dependency_scheduler_ids = plan.scheduler_dependencies(index, &scheduler_ids)?;

        let script_path = script_path_for(Path::new(&selected_job.planned.outputs.log));
        let script = render_hpc_submission_script(
            &report,
            selected_job.planned,
            &script_path.display().to_string(),
            &dependency_scheduler_ids,
            locks,
        )?;
        publish_submission_script(&script_path, &script)?;

        let scheduler_job_id = match settings.mode {
            SubmissionMode::Mock => format!("mock-{:04}", index + 1),
            SubmissionMode::Real => backend.submit_job(
                selected_job.planned,
                &script_path,
                &dependency_scheduler_ids,
                &submitted_at,
            )?,
        };

        write_operator_files(
            matches!(settings.mode, SubmissionMode::Mock),
            selected_job,
            &scheduler_job_id,
            &submitted_at,
            &redaction_needles,
        )?;
        let input = HpcBundleSubmission {
            campaign: &report,
            job: selected_job.planned,
            scheduler_job_id: &scheduler_job_id,
            submitted_at: &submitted_at,
            script_path: &script_path.display().to_string(),
        };
        let publisher =
            SubmissionBundlePublisher { locks, backend, redaction_needles: &redaction_needles };
        publisher.publish_primary(&input)?;
        publisher.encrypt_operator_outputs(&input)?;

        scheduler_ids.push(scheduler_job_id.clone());
        jobs_out.push(SubmittedJob {
            job_name: selected_job.name.to_string(),
            stage: selected_job.planned.stage.clone(),
            tool: selected_job.planned.tool.clone(),
            sample: selected_job.planned.sample.clone(),
            array_task: selected_job.planned.array_task,
            planned_job_id: selected_job.planned.job_id.clone(),
            scheduler_job_id,
            dependency_scheduler_ids,
            script_path: script_path.display().to_string(),
            log_path: selected_job.planned.outputs.log.clone(),
            out_path: selected_job.planned.outputs.out.clone(),
            err_path: selected_job.planned.outputs.err.clone(),
            results_path: selected_job.planned.outputs.results.clone(),
            code_path: selected_job.planned.outputs.code.clone(),
        });
    }

    let report = SlurmSubmissionReport {
        schema_version: SLURM_SUBMISSION_SCHEMA_VERSION,
        mode: match settings.mode {
            SubmissionMode::Mock => "mock".to_string(),
            SubmissionMode::Real => "real".to_string(),
        },
        campaign_id: report.campaign_id,
        domain: report.domain,
        submitted_at,
        jobs: jobs_out,
    };
    validate_hpc_submission_report(&report)?;
    Ok(report)
}
