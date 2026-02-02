use std::path::Path;

use bijux_core::{CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::CoverageEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Coverage.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &CoverageEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let outputs =
        crate::stages::support::audit_outputs(bijux_domain_bam::BamStage::Coverage, out_dir);
    let prefix = out_dir.join("coverage");
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: crate::tools::mosdepth::args(bam, &prefix, params),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![bijux_core::ArtifactRef {
                name: "bam".to_string(),
                path: bam.to_path_buf(),
            }],
            outputs,
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "bam": bam,
            "regions": params.regions,
            "depth_thresholds": params.depth_thresholds,
        }),
        effective_params: crate::stages::support::ensure_effective_params(
            serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    crate::stages::support::ensure_required_outputs(
        plan,
        &["coverage_report", "coverage_summary", "summary"],
    )
}
