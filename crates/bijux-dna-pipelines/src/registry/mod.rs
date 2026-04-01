//! Pipeline registry for FASTQ, BAM, and cross-domain profiles.

mod catalog;
mod families;
mod pipeline_id;
mod profile_lookup;

pub use catalog::PipelineRegistry;
pub use families::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
pub use pipeline_id::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};
pub use profile_lookup::profile_by_id;
