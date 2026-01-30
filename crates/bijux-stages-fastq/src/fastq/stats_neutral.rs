use std::path::Path;

use anyhow::Result;
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlan, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.stats_neutral";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a stats_neutral plan.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_neutral(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlan> {
    Ok(StagePlan {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: Vec::new(),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir
        }),
        aux_images: std::collections::BTreeMap::new(),
    })
}
