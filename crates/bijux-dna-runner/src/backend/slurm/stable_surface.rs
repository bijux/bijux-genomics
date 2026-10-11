//! Owner: bijux-dna-runner
//! Stable Slurm backend exports.

pub use super::benchmark::{prepare_benchmark_slurm_job, PreparedBenchmarkSlurmJob};
pub use super::client::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_batch, submit_slurm_job,
};
pub use super::contracts::{
    SlurmAccounting, SlurmBatchSubmission, SlurmClientPaths, SlurmEvidenceSource, SlurmJobHandle,
    SlurmJobObservation, SlurmJobState, SlurmSubmissionFailure, SlurmSubmitRequest,
    SLURM_JOB_HANDLE_SCHEMA_V1,
};
pub use super::corpus_materialization::{
    collect_corpus_materialization_slurm_job, load_corpus_materialization_slurm_job_handle,
    load_corpus_materialization_slurm_spec, load_prepared_corpus_materialization_slurm_job,
    observe_corpus_materialization_slurm_job, prepare_corpus_materialization_slurm_job,
    submit_prepared_corpus_materialization_job, CorpusMaterializationCollection,
    CorpusMaterializationEvidence, CorpusMaterializationEvidenceStatus,
    CorpusMaterializationSlurmResources, CorpusMaterializationSlurmSpec,
    PreparedCorpusMaterializationSlurmJob, CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1,
    CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1, CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};
pub use super::submission::{load_benchmark_slurm_job_handle, submit_prepared_benchmark_job};
