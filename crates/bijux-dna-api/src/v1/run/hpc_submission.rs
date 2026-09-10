//! Compose the benchmark submission ports with runner scheduler and encryption transports.
use super::{
    observe_hpc_code_freeze, submit_slurm_job, write_encrypted_bundle, BundleWriteRequest,
    SlurmClientPaths, SlurmDependencyCondition, SlurmSubmitRequest,
};
use anyhow::{Context, Result};
use bijux_dna_bench::HpcSubmissionBackend;
use bijux_dna_bench_model::{HpcBundleSubmission, HpcCodeFreezeObservation, PlannedJob};
use std::path::{Path, PathBuf};

/// Runner-backed capabilities with an explicit invocation working directory.
///
/// Stability: stable v1 adapter over the benchmark submission ports.
pub struct HpcSchedulerSubmissionBackend {
    pub paths: SlurmClientPaths,
    pub working_directory: PathBuf,
}

impl HpcSubmissionBackend for HpcSchedulerSubmissionBackend {
    fn submit_job(
        &self,
        job: &PlannedJob,
        script: &Path,
        dependencies: &[String],
        submitted_at: &str,
    ) -> Result<String> {
        let absolute = |value: &str| {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                self.working_directory.join(path)
            }
        };
        let stdout_path = absolute(&job.outputs.out);
        let stderr_path = absolute(&job.outputs.err);
        for path in [&stdout_path, &stderr_path] {
            let parent = path.parent().context("scheduler output path has no parent")?;
            bijux_dna_infra::ensure_dir(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let request = SlurmSubmitRequest {
            run_id: job.job_id.clone(),
            dependencies: dependencies.to_vec(),
            dependency_condition: SlurmDependencyCondition::AfterOk,
            // The campaign script owns its existing single-task array directive.
            array: None,
            script_path: script.to_path_buf(),
            working_directory: self.working_directory.clone(),
            stdout_path,
            stderr_path,
            submitted_at_utc: submitted_at.to_string(),
        };
        submit_slurm_job(&self.paths, &request).map(|handle| handle.job_id)
    }

    fn observe_code_freeze(&self, script: &Path) -> Result<HpcCodeFreezeObservation> {
        observe_hpc_code_freeze(script)
    }

    fn publish_bundle(
        &self,
        input: &HpcBundleSubmission<'_>,
        output: &Path,
        kind: &str,
        plaintext: &[u8],
    ) -> Result<()> {
        write_encrypted_bundle(&BundleWriteRequest {
            output_path: output,
            bundle_kind: kind,
            campaign_id: &input.campaign.campaign_id,
            domain: &input.campaign.domain,
            stage: &input.job.stage,
            tool: &input.job.tool,
            sample: &input.job.sample,
            planned_job_id: &input.job.job_id,
            scheduler_job_id: input.scheduler_job_id,
            submitted_at: input.submitted_at,
            backend: &input.campaign.security.encryption_backend,
            recipients: &input.campaign.security.encryption_recipients,
            plaintext,
        })
        .map(|_| ())
    }
}
