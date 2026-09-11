use crate::bam::assignment::declarations::NOT_APPLICABLE;
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub(in crate::bam::assignment) struct BamCorpusAssignmentContext {
    pub(in crate::bam::assignment) stage_evidence: BTreeMap<String, BamStageEvidence>,
}
#[derive(Debug, Clone)]
pub(in crate::bam::assignment) struct BamStageEvidence {
    pub(in crate::bam::assignment) sample_id: String,
    pub(in crate::bam::assignment) input_contract: String,
    pub(in crate::bam::assignment) benchmark_limits: String,
    pub(in crate::bam::assignment) required_assets: String,
    pub(in crate::bam::assignment) expected_outputs: String,
    pub(in crate::bam::assignment) skip_behavior: String,
}
impl BamCorpusAssignmentContext {
    pub(in crate::bam::assignment) fn evidence_for_stage(
        &self,
        stage_id: &str,
    ) -> BamStageEvidence {
        self.stage_evidence.get(stage_id).cloned().unwrap_or_else(BamStageEvidence::not_applicable)
    }
}
impl BamStageEvidence {
    pub(in crate::bam::assignment) fn not_applicable() -> Self {
        Self {
            sample_id: NOT_APPLICABLE.to_string(),
            input_contract: NOT_APPLICABLE.to_string(),
            benchmark_limits: NOT_APPLICABLE.to_string(),
            required_assets: NOT_APPLICABLE.to_string(),
            expected_outputs: NOT_APPLICABLE.to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        }
    }
}
