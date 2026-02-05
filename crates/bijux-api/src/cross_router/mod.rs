//! Cross-domain pipeline runner entrypoints.

pub(crate) const CROSS_STAGE_ID: &str = "cross.align_stub";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AlignmentBoundary {
    pub bam_path: String,
    #[serde(default)]
    pub bai_path: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub rg_policy: Option<String>,
    #[serde(default)]
    pub aligner_meta: Option<std::collections::BTreeMap<String, String>>,
}

mod bam_exec;
mod fastq_to_bam;
mod manifests;

pub use fastq_to_bam::run_fastq_to_bam_profile;
