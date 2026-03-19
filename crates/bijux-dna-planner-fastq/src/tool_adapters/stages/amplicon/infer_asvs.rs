use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_INFER_ASVS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(tool: &ToolExecutionSpecV1, reads: &Path, out_dir: &Path) -> Result<StagePlanV1> {
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
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
                ArtifactId::from_static("reads"),
                reads.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("asv_table_tsv"),
                    out_dir.join("asv_abundance.tsv"),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("asv_sequences_fasta"),
                    out_dir.join("asv_sequences.fasta"),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fasta"),
                    out_dir.join("taxonomy_ready.fasta"),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("taxonomy_ready_fastq"),
                    out_dir.join("taxonomy_ready.fastq"),
                    ArtifactRole::Reads,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "requires_r_runtime": true,
            "output_table_kind": "asv_abundance_table",
            "runtime_constraints": {
                "requires_r": true,
                "min_r_major": 4
            }
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon ASV inference",
        ),
    })
}
