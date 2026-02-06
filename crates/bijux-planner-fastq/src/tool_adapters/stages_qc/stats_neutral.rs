use std::path::Path;

use anyhow::Result;
use bijux_core::{StageId, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{validate::ValidateEffectiveParams, PairedMode};
use bijux_domain_fastq::STAGE_STATS_NEUTRAL;
use bijux_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_STATS_NEUTRAL;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a stats_neutral plan.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_neutral(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let effective_params = ValidateEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        q_cutoff: None,
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                "reads_r1",
                r1.to_path_buf(),
                bijux_core::ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                "stats_json",
                out_dir.join("stats.json"),
                bijux_core::ArtifactRole::MetricsJson,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize stats_neutral effective params"),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    })
}
