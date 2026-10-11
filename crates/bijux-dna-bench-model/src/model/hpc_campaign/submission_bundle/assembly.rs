//! Typed assembly of submission records from planned inputs and captured observations.
use super::inputs::{HpcBundleFoundationLocks, HpcBundleSubmission, HpcCodeFreezeObservation};
use super::wire::{
    BundleAppraiser, BundleArtifact, BundleArtifacts, BundleCampaign, BundleCodeCampaign,
    BundleCodeFreeze, BundleConfigReferences, BundleDvcState, BundleEffectiveSettings,
    BundleInventories, BundleJob, BundleLocks, BundleMetrics, BundlePlan, BundleReport,
    BundleRepositoryState, BundleResultsJob, BundleTraces, CodeBundleVersion,
    HpcCodeSubmissionBundle, HpcResultsSubmissionBundle, ResultsBundleVersion,
};
use sha2::{Digest, Sha256};

/// Assemble the existing results submission format without claiming completed execution.
#[must_use]
pub fn assemble_hpc_results_submission(
    input: &HpcBundleSubmission<'_>,
) -> HpcResultsSubmissionBundle {
    let report = input.campaign;
    let job = input.job;
    let artifact = |kind: &str, path: &str| BundleArtifact { kind: kind.into(), path: path.into() };
    HpcResultsSubmissionBundle {
        schema_version: ResultsBundleVersion::V1,
        campaign: BundleCampaign {
            id: report.campaign_id.clone(),
            domain: report.domain.clone(),
            config_path: report.config_path.clone(),
            env_file_path: report.env_file_path.clone(),
            user_policy_path: report.user_policy_path.clone(),
        },
        job: BundleResultsJob {
            planned_job_id: job.job_id.clone(),
            scheduler_job_id: input.scheduler_job_id.into(),
            submitted_at: input.submitted_at.into(),
            job_name: job.job_name.clone(),
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            resource_template: job.resource_template.clone(),
            resources: (&job.resources).into(),
            depends_on: job.depends_on.clone(),
        },
        metrics: BundleMetrics {
            status: "pending_execution".into(),
            submission_mode: "slurm".into(),
        },
        artifacts: BundleArtifacts {
            inventory: vec![
                artifact("log", &job.outputs.log),
                artifact("out", &job.outputs.out),
                artifact("err", &job.outputs.err),
            ],
            encrypted_targets: vec![
                artifact("results", &job.outputs.results),
                artifact("code", &job.outputs.code),
            ],
        },
        reports: vec![BundleReport {
            kind: "submission_receipt".into(),
            summary: "job submitted or mocked; execution metrics pending runtime wrapper output"
                .into(),
        }],
        traces: BundleTraces {
            script_path: input.script_path.into(),
            scheduler_job_id: input.scheduler_job_id.into(),
        },
        inventories: BundleInventories {
            encryption_backend: report.security.encryption_backend.clone(),
            recipient_fingerprints: report
                .security
                .encryption_recipients
                .iter()
                .map(|recipient| {
                    use std::fmt::Write as _;
                    let mut fingerprint = String::with_capacity(16);
                    for byte in Sha256::digest(recipient.as_bytes()).iter().take(8) {
                        let _ = write!(&mut fingerprint, "{byte:02x}");
                    }
                    fingerprint
                })
                .collect(),
        },
        appraiser_outputs: vec![BundleAppraiser {
            name: "submission_ready".into(),
            status: "pending".into(),
            note: "appraiser jobs are tracked after runtime completion".into(),
        }],
    }
}

/// Assemble the existing code submission format from explicit current observations.
#[must_use]
pub fn assemble_hpc_code_submission(
    input: &HpcBundleSubmission<'_>,
    observation: HpcCodeFreezeObservation,
    locks: HpcBundleFoundationLocks,
) -> HpcCodeSubmissionBundle {
    let report = input.campaign;
    let job = input.job;
    HpcCodeSubmissionBundle {
        schema_version: CodeBundleVersion::V1,
        campaign: BundleCodeCampaign {
            id: report.campaign_id.clone(),
            domain: report.domain.clone(),
            config_path: report.config_path.clone(),
            env_file_path: report.env_file_path.clone(),
            user_policy_path: report.user_policy_path.clone(),
            user_policies_applied: report.user_policies_applied,
        },
        job: BundleJob {
            planned_job_id: job.job_id.clone(),
            scheduler_job_id: input.scheduler_job_id.into(),
            submitted_at: input.submitted_at.into(),
            job_name: job.job_name.clone(),
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
        },
        code_freeze: BundleCodeFreeze {
            slurm_script: observation.slurm_script,
            effective_settings: BundleEffectiveSettings {
                slurm: (&report.resolved_slurm).into(),
                resources: (&job.resources).into(),
            },
            config_references: BundleConfigReferences {
                campaign_config: report.config_path.clone(),
                env_file: report.env_file_path.clone(),
                user_policy: report.user_policy_path.clone(),
            },
            repository_state: BundleRepositoryState {
                head: observation.git_head,
                branch: observation.git_branch,
                status_porcelain: observation.git_status_porcelain,
            },
            dvc_state: BundleDvcState {
                available: observation.dvc_available,
                status_hint: "capture deferred to runtime wrapper when dvc is configured".into(),
            },
        },
        locks: BundleLocks {
            corpus: locks.corpus_lock,
            database: locks.database_lock,
            image: locks.image_lock,
            tool: "deferred_to_runtime_registry_capture".into(),
        },
        plan: BundlePlan { depends_on: job.depends_on.clone(), outputs: (&job.outputs).into() },
    }
}
