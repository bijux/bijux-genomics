use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PROFILE_READ_LENGTHS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a pre-trim length distribution plan.
///
/// # Errors
/// Returns an error if plan serialization fails.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let dist_tsv = out_dir.join("length_distribution.tsv");
    let dist_json = out_dir.join("length_distribution.json");
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
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
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("length_distribution_tsv"),
                    dist_tsv.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("length_distribution_json"),
                    dist_json.clone(),
                    ArtifactRole::MetricsJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_tsv": dist_tsv,
            "output_json": dist_json,
        }),
        effective_params: serde_json::json!({
            "stage": "profile_read_lengths",
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "threads": tool.resources.threads,
            "schema": ["sample_id", "read_length", "count"],
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::new(
            bijux_dna_stage_contract::PlanReasonKind::Default,
            "pre-trim length distribution metrics",
        ),
    })
}
