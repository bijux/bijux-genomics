//! Owner: bijux-dna-runner
//! Governed container qualification Slurm lifecycle.

mod archival;
mod contracts;
mod cycle;
mod execution;
mod lifecycle;
mod preparation;
mod validation;

pub use archival::{
    archive_failed_container_qualification_attempt,
    rollback_failed_container_qualification_attempt_archive,
};
pub use contracts::{
    ContainerQualificationAccountingRecord, ContainerQualificationAdmission,
    ContainerQualificationCollection, ContainerQualificationExecution,
    ContainerQualificationPreparationRequest, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmResources,
    ContainerQualificationSlurmSpec, ContainerQualificationSubmissionKind,
    HistoricalContainerQualificationReplayRequest, HistoricalContainerQualificationReplaySource,
    PreparedContainerQualificationSlurmJob, CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1,
    CONTAINER_QUALIFICATION_ADMISSION_SCHEMA_V1, CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
};
pub use cycle::{
    admit_container_qualification_cycle, advance_container_qualification_cycle,
    collect_container_qualification_cycle, load_container_qualification_cycle_preparation,
    plan_container_qualification_cycle, status_container_qualification_cycle,
    ContainerQualificationCycleAction, ContainerQualificationCyclePreparation,
    ContainerQualificationCycleRequest, ContainerQualificationCycleState,
    ContainerQualificationCycleStatus, ContainerQualificationCycleSummary,
    ContainerQualificationCycleUnit, ContainerQualificationSourceMaterialization,
    CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1,
    CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2,
    CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1, CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2,
};
pub use execution::execute_container_qualification;
pub use lifecycle::{
    admit_container_qualification_collection, cancel_container_qualification_slurm_job,
    collect_container_qualification_slurm_job, load_container_qualification_admission,
    load_container_qualification_scheduler_evidence, load_container_qualification_slurm_job_handle,
    load_prepared_container_qualification_slurm_job, observe_container_qualification_slurm_job,
    submit_prepared_container_qualification_job,
};
pub use preparation::{
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
};
pub use validation::{
    load_container_qualification_slurm_spec, load_recorded_container_qualification_slurm_spec,
};
