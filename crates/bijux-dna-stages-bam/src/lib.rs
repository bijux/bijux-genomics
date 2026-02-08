//! BAM stage specs, metrics, and observers.

pub mod metrics;
pub mod observer;
mod plugin;
pub mod stage_specs;

pub use bijux_dna_stage_contract::StagePlanJsonV1 as StagePlanJson;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_dna_domain_bam::BamStage> {
    vec![
        bijux_dna_domain_bam::BamStage::Align,
        bijux_dna_domain_bam::BamStage::Validate,
        bijux_dna_domain_bam::BamStage::QcPre,
        bijux_dna_domain_bam::BamStage::Filter,
        bijux_dna_domain_bam::BamStage::Markdup,
        bijux_dna_domain_bam::BamStage::Complexity,
        bijux_dna_domain_bam::BamStage::Coverage,
        bijux_dna_domain_bam::BamStage::Damage,
        bijux_dna_domain_bam::BamStage::Authenticity,
        bijux_dna_domain_bam::BamStage::Contamination,
        bijux_dna_domain_bam::BamStage::Sex,
        bijux_dna_domain_bam::BamStage::BiasMitigation,
        bijux_dna_domain_bam::BamStage::Recalibration,
        bijux_dna_domain_bam::BamStage::Haplogroups,
        bijux_dna_domain_bam::BamStage::Genotyping,
        bijux_dna_domain_bam::BamStage::Kinship,
    ]
}
