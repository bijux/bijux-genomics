//! Stage specs, metrics, and observers for FASTQ.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_core::ids::StageId> {
    vec![
        bijux_dna_domain_fastq::STAGE_VALIDATE_PRE,
        bijux_dna_domain_fastq::stages::STAGE_PREPARE_REFERENCE,
        bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS,
        bijux_dna_domain_fastq::STAGE_TRIM,
        bijux_dna_domain_fastq::STAGE_FILTER,
        bijux_dna_domain_fastq::STAGE_STATS_NEUTRAL,
        bijux_dna_domain_fastq::stages::STAGE_RRNA,
        bijux_dna_domain_fastq::STAGE_MERGE,
        bijux_dna_domain_fastq::STAGE_CORRECT,
        bijux_dna_domain_fastq::STAGE_UMI,
        bijux_dna_domain_fastq::STAGE_SCREEN,
        bijux_dna_domain_fastq::STAGE_QC_POST,
        bijux_dna_domain_fastq::STAGE_PREPROCESS,
    ]
}

pub mod contracts {
    pub use bijux_dna_domain_fastq::contract_for_stage;
    pub use bijux_dna_domain_fastq::FastqStageContract as StageContract;
}
