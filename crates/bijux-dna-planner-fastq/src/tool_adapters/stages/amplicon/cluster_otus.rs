use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_CLUSTER_OTUS;
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
                    ArtifactId::from_static("otu_table_tsv"),
                    out_dir.join("otu_abundance.tsv"),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("otu_sequences_fasta"),
                    out_dir.join("otu_representatives.fasta"),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("merged_reads"),
                    out_dir.join("merged_reads.fastq.gz"),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("unmerged_reads_r1"),
                    out_dir.join("unmerged_reads_R1.fastq.gz"),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("unmerged_reads_r2"),
                    out_dir.join("unmerged_reads_R2.fastq.gz"),
                    ArtifactRole::Reads,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({
            "identity_threshold": 0.97,
            "output_table_kind": "otu_abundance_table",
            "output_naming": "deterministic"
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "amplicon OTU clustering",
        ),
    })
}
