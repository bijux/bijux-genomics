use super::wire::{BundleOutputs, BundleResources, BundleSlurm};

impl From<&crate::ResourceTemplate> for BundleResources {
    fn from(value: &crate::ResourceTemplate) -> Self {
        Self {
            cpus: value.cpus,
            mem_gb: value.mem_gb,
            walltime: value.walltime.clone(),
            scratch_gb: value.scratch_gb,
        }
    }
}

impl From<&crate::ResolvedSlurm> for BundleSlurm {
    fn from(value: &crate::ResolvedSlurm) -> Self {
        Self {
            site_profile: value.site_profile.clone(),
            account_redacted: value.account_redacted.clone(),
            project_redacted: value.project_redacted.clone(),
            partition: value.partition.clone(),
            qos: value.qos.clone(),
            default_resource_template: value.default_resource_template.clone(),
            retry_attempts: value.retry_attempts,
            retry_backoff_seconds: value.retry_backoff_seconds,
            retry_on_exit_codes: value.retry_on_exit_codes.clone(),
        }
    }
}

impl From<&crate::PlannedOutputs> for BundleOutputs {
    fn from(value: &crate::PlannedOutputs) -> Self {
        Self {
            scratch_dir: value.scratch_dir.clone(),
            log: value.log.clone(),
            out: value.out.clone(),
            err: value.err.clone(),
            results: value.results.clone(),
            code: value.code.clone(),
        }
    }
}
