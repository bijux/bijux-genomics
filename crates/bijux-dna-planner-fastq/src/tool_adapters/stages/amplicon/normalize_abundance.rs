use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::edna::AbundanceNormalizationEffectiveParams;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_NORMALIZE_ABUNDANCE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(
    tool: &ToolExecutionSpecV1,
    abundance_table: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("abundance_table"),
                abundance_table.to_path_buf(),
                ArtifactRole::SummaryTsv,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("normalized_abundance_tsv"),
                out_dir.join("abundance_normalized.tsv"),
                ArtifactRole::SummaryTsv,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::to_value(AbundanceNormalizationEffectiveParams {
            method: "relative_abundance".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            normalized_value_column: "normalized_abundance".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
        })?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon abundance normalization",
        ),
    })
}
