use std::path::Path;

use bijux_core::{StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::BiasMitigationEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::BiasMitigation.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &BiasMitigationEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let outputs = super::audit_outputs(bijux_domain_bam::BamStage::BiasMitigation, out_dir);
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
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
            "gc_bias_correction": params.gc_bias_correction,
            "map_bias_correction": params.map_bias_correction,
        }),
        effective_params: super::ensure_effective_params(
            serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    super::ensure_required_outputs(plan, &["bias_report", "summary"])
}
