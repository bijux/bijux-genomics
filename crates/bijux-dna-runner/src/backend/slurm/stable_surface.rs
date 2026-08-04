//! Owner: bijux-dna-runner
//! Stable Slurm backend exports.

pub use super::client::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_batch, submit_slurm_job,
};
pub use super::contracts::{
    SlurmAccounting, SlurmBatchSubmission, SlurmClientPaths, SlurmEvidenceSource, SlurmJobHandle,
    SlurmJobObservation, SlurmJobState, SlurmSubmissionFailure, SlurmSubmitRequest,
    SLURM_JOB_HANDLE_SCHEMA_V1,
};
