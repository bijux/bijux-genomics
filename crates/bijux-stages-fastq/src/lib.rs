//! Stage specs, metrics, and observers for FASTQ.

pub mod metrics;
pub mod observer;
pub mod observer_artifacts;
mod plugin;
pub mod stage_specs;

pub use bijux_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_core::ids::StageId> {
    vec![
        bijux_domain_fastq::STAGE_VALIDATE_PRE,
        bijux_domain_fastq::STAGE_DETECT_ADAPTERS,
        bijux_domain_fastq::STAGE_TRIM,
        bijux_domain_fastq::STAGE_FILTER,
        bijux_domain_fastq::STAGE_STATS_NEUTRAL,
        bijux_domain_fastq::STAGE_MERGE,
        bijux_domain_fastq::STAGE_CORRECT,
        bijux_domain_fastq::STAGE_UMI,
        bijux_domain_fastq::STAGE_SCREEN,
        bijux_domain_fastq::STAGE_QC_POST,
        bijux_domain_fastq::STAGE_PREPROCESS,
    ]
}

pub mod contracts {
    pub use bijux_domain_fastq::contract_for_stage;
    pub use bijux_domain_fastq::FastqStageContract as StageContract;
}
