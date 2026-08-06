//! Owner: bijux-dna-runner
//! Governed FASTQ corpus materialization Slurm lifecycle.

mod contracts;
mod lifecycle;
mod preparation;
mod validation;

pub use contracts::{
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmResources,
    CorpusMaterializationSlurmSpec, PreparedCorpusMaterializationSlurmJob,
    CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1, CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};
pub use lifecycle::{
    collect_corpus_materialization_slurm_job, load_corpus_materialization_slurm_job_handle,
    load_prepared_corpus_materialization_slurm_job, observe_corpus_materialization_slurm_job,
    submit_prepared_corpus_materialization_job,
};
pub use preparation::prepare_corpus_materialization_slurm_job;
pub use validation::load_corpus_materialization_slurm_spec;
