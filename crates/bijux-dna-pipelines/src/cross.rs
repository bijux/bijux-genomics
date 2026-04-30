//! Cross-domain pipeline profiles.

mod bam_to_vcf;
mod fastq_to_bam;
mod fastq_to_vcf;
mod workflow_registry;

pub use bam_to_vcf::bam_to_vcf_default_profile;
pub use fastq_to_bam::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
pub use fastq_to_vcf::fastq_to_vcf_minimal_profile;
pub use workflow_registry::{
    cross_workflow_template_by_id, cross_workflow_templates, cross_workflow_templates_for_pipeline,
};
