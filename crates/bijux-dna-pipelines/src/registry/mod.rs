//! Pipeline registry for FASTQ, BAM, and cross-domain profiles.

mod catalog;
pub mod id;
mod lookup;

pub use catalog::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles, PipelineRegistry};
pub use lookup::profile_by_id;
