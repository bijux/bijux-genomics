//! BAM stage specs, metrics, and observers.

pub mod metrics;
pub mod observer;
pub mod plugin;
pub mod stage_specs;

pub use bijux_core::StagePlanJsonV1 as StagePlanJson;

pub use bijux_domain_bam as domain_bam;

#[must_use]
pub fn implemented_stages() -> Vec<bijux_domain_bam::BamStage> {
    vec![
        bijux_domain_bam::BamStage::Align,
        bijux_domain_bam::BamStage::Validate,
        bijux_domain_bam::BamStage::QcPre,
        bijux_domain_bam::BamStage::Filter,
        bijux_domain_bam::BamStage::Markdup,
        bijux_domain_bam::BamStage::Complexity,
        bijux_domain_bam::BamStage::Coverage,
        bijux_domain_bam::BamStage::Damage,
        bijux_domain_bam::BamStage::Authenticity,
        bijux_domain_bam::BamStage::Contamination,
        bijux_domain_bam::BamStage::Sex,
        bijux_domain_bam::BamStage::BiasMitigation,
        bijux_domain_bam::BamStage::Recalibration,
        bijux_domain_bam::BamStage::Haplogroups,
        bijux_domain_bam::BamStage::Genotyping,
        bijux_domain_bam::BamStage::Kinship,
    ]
}
