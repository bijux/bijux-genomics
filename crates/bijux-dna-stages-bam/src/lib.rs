//! BAM stage specs, metrics, and observers.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_domain_bam::BamStage> {
    bijux_dna_domain_bam::BamStage::all().to_vec()
}
