//! Owner: bijux-dna-runner
//! Governed container qualification Slurm lifecycle.

mod archival;
mod contracts;
mod execution;
mod lifecycle;
mod preparation;
mod validation;

pub use archival::archive_failed_container_qualification_attempt;
pub use contracts::{
    ArchivedContainerQualificationAttempt, ContainerQualificationAccountingRecord,
    ContainerQualificationArchiveFile, ContainerQualificationArchiveSources,
    ContainerQualificationCollection, ContainerQualificationExecution,
    ContainerQualificationFailedAttemptArchive, ContainerQualificationPreparationRequest,
    ContainerQualificationSchedulerEvidence, ContainerQualificationSchedulerStatus,
    ContainerQualificationSlurmResources, ContainerQualificationSlurmSpec,
    ContainerQualificationSubmissionKind, HistoricalContainerQualificationReplayRequest,
    HistoricalContainerQualificationReplaySource, PreparedContainerQualificationSlurmJob,
    CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1, CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_FAILED_ATTEMPT_ARCHIVE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
};
pub use execution::execute_container_qualification;
pub use lifecycle::{
    cancel_container_qualification_slurm_job, collect_container_qualification_slurm_job,
    load_container_qualification_slurm_job_handle, load_prepared_container_qualification_slurm_job,
    observe_container_qualification_slurm_job, submit_prepared_container_qualification_job,
};
pub use preparation::{
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
};
pub use validation::load_container_qualification_slurm_spec;
