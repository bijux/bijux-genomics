use super::super::tool_serving_map::{
    render_fastq_tool_serving_map, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use anyhow::Result;
use bijux_dna_bench_fastq::readiness::declarations::{
    observations::{OutputAssessment, OutputDeclaration, ServingBinding},
    ports::OutputEvidence,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_planner_fastq::stage_api::{
    assess_fastq_output_declarations, load_fastq_domain_tool_stage_output_contract,
};
use std::path::Path;
pub(super) struct OutputAdapter;
impl OutputEvidence for OutputAdapter {
    fn serving(&self, root: &Path) -> Result<Vec<ServingBinding>> {
        Ok(render_fastq_tool_serving_map(root, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH.into())?
            .rows
            .into_iter()
            .map(|row| ServingBinding {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                adapter_status: row.adapter_status,
            })
            .collect())
    }
    fn assess(
        &self,
        root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<(OutputDeclaration, OutputAssessment)> {
        let contract = load_fastq_domain_tool_stage_output_contract(
            root,
            &StageId::new(stage_id.to_string()),
            &ToolId::new(tool_id.to_string()),
        )?;
        let assessment = assess_fastq_output_declarations(&contract)?;
        let complete = assessment.is_complete();
        Ok((
            OutputDeclaration {
                declared_output_ids: contract.declared_output_ids,
                execution_expected_output_ids: contract.execution_expected_output_ids,
                stage_expected_artifact_ids: contract.stage_expected_artifact_ids,
            },
            OutputAssessment {
                complete,
                normalized_metrics_output_id: assessment.normalized_metrics_output_id,
                raw_output_artifact_ids: assessment.raw_output_artifact_ids,
                missing_declarations: assessment.missing_declarations,
            },
        ))
    }
}
