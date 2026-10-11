//! Genotype benchmark wire reports and admitted observations.

pub(crate) const GOVERNED_VCF_CALL_TOOL_ID: &str = "bcftools";
pub(crate) const GOVERNED_BAM_SAMPLE_ID: &str = "human_like_validation";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfCallBamSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_bam: String,
    pub(crate) input_bam_index: String,
    pub(crate) reference: String,
    pub(crate) sample_id: String,
    pub(crate) sample_name: String,
}
