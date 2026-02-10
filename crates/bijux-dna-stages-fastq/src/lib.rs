//! Stage specs, metrics, and observers for FASTQ.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_core::ids::StageId> {
    bijux_dna_domain_fastq::STAGES.to_vec()
}

pub mod contracts {
    pub use bijux_dna_domain_fastq::contract_for_stage;
    pub use bijux_dna_domain_fastq::FastqStageContract as StageContract;
}
