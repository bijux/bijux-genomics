//! Admit selected campaign context before script or scheduler effects.
use super::HpcSubmissionPlan;
use crate::CampaignDryRunReport;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

/// Validate invocation identity and every selected job before any submission effects.
/// # Errors
/// Rejects empty campaign fields, empty selected identities/output paths and duplicate planned IDs.
pub fn validate_hpc_submission_request(
    report: &CampaignDryRunReport,
    plan: &HpcSubmissionPlan<'_>,
    submitted_at: &str,
) -> Result<()> {
    ensure!(
        [report.campaign_id.as_str(), report.domain.as_str(), submitted_at]
            .iter()
            .all(|value| !value.trim().is_empty()),
        "submission context has an empty campaign identity or timestamp"
    );
    let mut planned = BTreeSet::new();
    for selected in plan.jobs() {
        let job = selected.planned;
        ensure!(
            [
                job.job_name.as_str(),
                job.job_id.as_str(),
                job.stage.as_str(),
                job.tool.as_str(),
                job.sample.as_str(),
                job.outputs.log.as_str(),
                job.outputs.out.as_str(),
                job.outputs.err.as_str(),
                job.outputs.results.as_str(),
                job.outputs.code.as_str(),
            ]
            .iter()
            .all(|value| !value.trim().is_empty()),
            "submission context has an empty selected job identity or output path"
        );
        ensure!(
            planned.insert(job.job_id.as_str()),
            "submission context repeats a planned job identity"
        );
    }
    Ok(())
}
