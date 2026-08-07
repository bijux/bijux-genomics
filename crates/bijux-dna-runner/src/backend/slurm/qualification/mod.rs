//! Owner: bijux-dna-runner
//! Governed container qualification Slurm lifecycle.

mod contracts;
mod lifecycle;
mod preparation;
mod validation;

pub use contracts::{
    ContainerQualificationAccountingRecord, ContainerQualificationCollection,
    ContainerQualificationPreparationRequest, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmResources,
    ContainerQualificationSlurmSpec, ContainerQualificationSubmissionKind,
    HistoricalContainerQualificationReplayRequest, HistoricalContainerQualificationReplaySource,
    PreparedContainerQualificationSlurmJob, CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1,
    CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1, CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2,
};
pub use lifecycle::{
    cancel_container_qualification_slurm_job, collect_container_qualification_slurm_job,
    load_container_qualification_slurm_job_handle, load_prepared_container_qualification_slurm_job,
    observe_container_qualification_slurm_job, submit_prepared_container_qualification_job,
};
pub use preparation::{
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
};
pub use validation::load_container_qualification_slurm_spec;
