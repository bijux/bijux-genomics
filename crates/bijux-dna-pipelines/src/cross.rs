//! Cross-domain pipeline profiles.

use bijux_dna_domain_bam::defaults::default_params_json;
use bijux_dna_domain_bam::BamStage;

mod bam_to_vcf;
mod fastq_to_bam;
mod fastq_to_vcf;
mod workflow_registry;

use crate::DefaultParams;

pub use bam_to_vcf::bam_to_vcf_default_profile;
pub use fastq_to_bam::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
pub use fastq_to_vcf::fastq_to_vcf_minimal_profile;
pub use workflow_registry::{
    cross_workflow_template_by_id, cross_workflow_templates, cross_workflow_templates_for_pipeline,
};

fn bam_genotyping_params() -> DefaultParams {
    DefaultParams::Bam(
        BamStage::Genotyping
            .parse_effective_params(&default_params_json(BamStage::Genotyping))
            .unwrap_or_else(|err| panic!("failed to parse BAM genotyping defaults: {err}")),
    )
}
