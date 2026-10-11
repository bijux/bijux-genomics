//! Owner: bijux-dna-runner
//! Restart-safe one-shot Slurm lifecycle for immutable external workers.

mod contracts;
mod lifecycle;
mod preparation;
mod validation;

pub use contracts::{
    ManagedSlurmJobCollection, ManagedSlurmJobEvidence, ManagedSlurmJobResources,
    ManagedSlurmJobSpec, PreparedManagedSlurmJob, MANAGED_SLURM_JOB_EVIDENCE_SCHEMA_V1,
    MANAGED_SLURM_JOB_SPEC_SCHEMA_V1, PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1,
};
pub use lifecycle::{
    collect_managed_slurm_job, load_managed_slurm_job_handle, load_prepared_managed_slurm_job,
    observe_managed_slurm_job, submit_prepared_managed_slurm_job,
};
pub use preparation::prepare_managed_slurm_job;
pub use validation::load_managed_slurm_job_spec;
