//! Owner: bijux-dna-runner
//! Stable Slurm backend exports.

pub use super::benchmark::{prepare_benchmark_slurm_job, PreparedBenchmarkSlurmJob};
pub use super::client::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_batch, submit_slurm_job,
};
pub use super::contracts::{
    SlurmAccounting, SlurmArraySpec, SlurmArrayTaskAccounting, SlurmBatchSubmission,
    SlurmClientPaths, SlurmDependencyCondition, SlurmEvidenceSource, SlurmJobHandle,
    SlurmJobObservation, SlurmJobState, SlurmSubmissionFailure, SlurmSubmitRequest,
    SLURM_JOB_HANDLE_SCHEMA_V1,
};
pub use super::corpus_materialization::{
    collect_corpus_materialization_slurm_job, load_corpus_materialization_slurm_job_handle,
    load_corpus_materialization_slurm_spec, load_prepared_corpus_materialization_slurm_job,
    observe_corpus_materialization_slurm_job, prepare_corpus_materialization_slurm_job,
    publish_corpus_materialization_evidence_correction, submit_prepared_corpus_materialization_job,
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceCorrection, CorpusMaterializationEvidenceCorrectionRequest,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmResources,
    CorpusMaterializationSlurmSpec, PreparedCorpusMaterializationSlurmJob,
    CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1,
    CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1, CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};
pub use super::qualification::{
    archive_failed_container_qualification_attempt, cancel_container_qualification_slurm_job,
    collect_container_qualification_slurm_job, execute_container_qualification,
    load_container_qualification_slurm_job_handle, load_container_qualification_slurm_spec,
    load_prepared_container_qualification_slurm_job, observe_container_qualification_slurm_job,
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
    submit_prepared_container_qualification_job, ArchivedContainerQualificationAttempt,
    ContainerQualificationAccountingRecord, ContainerQualificationArchiveFile,
    ContainerQualificationArchiveSources, ContainerQualificationCollection,
    ContainerQualificationExecution, ContainerQualificationFailedAttemptArchive,
    ContainerQualificationPreparationRequest, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmResources,
    ContainerQualificationSlurmSpec, ContainerQualificationSubmissionKind,
    HistoricalContainerQualificationReplayRequest, HistoricalContainerQualificationReplaySource,
    PreparedContainerQualificationSlurmJob, CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1,
    CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_FAILED_ATTEMPT_ARCHIVE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4,
};
pub use super::submission::{load_benchmark_slurm_job_handle, submit_prepared_benchmark_job};
