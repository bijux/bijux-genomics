use std::path::Path;

use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::ComplexityEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Complexity.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &ComplexityEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "bam".to_string(),
                path: bam.to_path_buf(),
            }],
            outputs: vec![
                ArtifactRef {
                    name: "complexity_report".to_string(),
                    path: out_dir.join("complexity.json"),
                },
                ArtifactRef {
                    name: "complexity_preseq".to_string(),
                    path: out_dir.join("preseq.txt"),
                },
                ArtifactRef {
                    name: "complexity_metrics".to_string(),
                    path: out_dir.join("complexity.metrics.json"),
                },
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "bam": bam,
            "min_reads": params.min_reads,
            "projection_points": params.projection_points,
        }),
        effective_params: serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        aux_images: std::collections::BTreeMap::new(),
    };
    super::ensure_required_outputs(
        plan,
        &[
            "complexity_report",
            "complexity_preseq",
            "complexity_metrics",
        ],
    )
}
