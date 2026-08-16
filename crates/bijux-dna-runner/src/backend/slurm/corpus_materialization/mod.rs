//! Owner: bijux-dna-runner
//! Governed corpus materialization Slurm lifecycle.

mod contracts;
mod correction;
mod lifecycle;
mod planning;
mod preparation;
mod validation;

pub use contracts::{
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceCorrection, CorpusMaterializationEvidenceCorrectionRequest,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmPlanRequest,
    CorpusMaterializationSlurmResources, CorpusMaterializationSlurmSpec,
    PreparedCorpusMaterializationSlurmJob, CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1,
    CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1, CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V2,
    CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1, CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V2,
};
pub use correction::publish_corpus_materialization_evidence_correction;
pub use lifecycle::{
    collect_corpus_materialization_slurm_job, load_corpus_materialization_slurm_job_handle,
    load_prepared_corpus_materialization_slurm_job, observe_corpus_materialization_slurm_job,
    submit_prepared_corpus_materialization_job,
};
pub use planning::plan_corpus_materialization_slurm_spec;
pub use preparation::prepare_corpus_materialization_slurm_job;
pub use validation::load_corpus_materialization_slurm_spec;
