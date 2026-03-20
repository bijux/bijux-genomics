//! Stage specs, metrics, and observers for FASTQ.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn contract_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::STAGES.to_vec()
}

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_core::ids::StageId> {
    closed_execution_stage_ids()
}

#[must_use]
pub fn closed_execution_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::execution_closed_stage_ids()
}

#[must_use]
pub fn observer_stage_ids() -> Vec<bijux_dna_core::ids::StageId> {
    vec![
        bijux_dna_domain_fastq::STAGE_VALIDATE_READS,
        bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS,
        bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS,
        bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
        bijux_dna_domain_fastq::STAGE_PROFILE_READS,
        bijux_dna_domain_fastq::STAGE_REPORT_QC,
    ]
}

pub mod contracts {
    pub use bijux_dna_domain_fastq::contract_for_stage;
    pub use bijux_dna_domain_fastq::FastqStageContract as StageContract;
}
