//! Validated imputation variant and masked truth evidence.

pub(crate) struct ImputationEvidence {
    pub(crate) validation: bijux_dna_stages_vcf::vcf_io::VcfValidationSummary,
    pub(crate) output_summary: crate::imputation::model::ImputeOutputSummary,
    pub(crate) quality: super::quality::ImputationQuality,
    pub(crate) sample_count: u64,
}
