//! Pipeline registry for FASTQ, BAM, and cross-domain profiles.

mod catalog;
mod pipeline_id;
mod profile_collections;
mod profile_lookup;

pub use catalog::PipelineRegistry;
pub use pipeline_id::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};
pub use profile_collections::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
pub use profile_lookup::profile_by_id;
