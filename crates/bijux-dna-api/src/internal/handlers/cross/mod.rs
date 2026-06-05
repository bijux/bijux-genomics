//! Cross-domain pipeline runner entrypoints.

mod bam_exec;
mod bam_normalized_metrics_contract;
mod fastq_to_bam;
mod manifests;

pub(crate) use crate::internal::alignment_boundary::AlignmentBoundary;
pub(crate) fn render_governed_bam_normalized_metrics_schema() -> serde_json::Value {
    bam_normalized_metrics_contract::render_bam_normalized_metrics_schema()
}

pub use fastq_to_bam::run_fastq_to_bam_profile;
