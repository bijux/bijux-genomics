use super::BamEvidenceAdapter;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::{observations::OutputAssessment, ports::OutputEvidence};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::stage_specs::stage_output_ids_in_manifest_order;
use bijux_dna_planner_bam::stage_api::{
    assess_bam_output_declarations, load_bam_domain_tool_stage_output_contract,
};
use std::path::Path;
impl OutputEvidence for BamEvidenceAdapter {
    fn stage_output_ids(&self, _repo_root: &Path, stage_id: &str) -> Vec<String> {
        stage_output_ids_in_manifest_order(stage_id).unwrap_or_default()
    }
    fn assess_output(
        &self,
        repo_root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<OutputAssessment> {
        let contract = load_bam_domain_tool_stage_output_contract(
            repo_root,
            &StageId::new(stage_id),
            &ToolId::new(tool_id),
        )?;
        let assessment = assess_bam_output_declarations(&contract)?;
        let complete = assessment.is_complete();
        Ok(OutputAssessment {
            complete,
            declared_output_ids: contract.declared_output_ids,
            execution_expected_output_ids: contract.execution_expected_output_ids,
            stage_expected_artifact_ids: contract.stage_expected_artifact_ids,
            normalized_metrics_output_id: assessment.normalized_metrics_output_id,
            raw_output_artifact_ids: assessment.raw_output_artifact_ids,
            missing_declarations: assessment.missing_declarations,
        })
    }
}
