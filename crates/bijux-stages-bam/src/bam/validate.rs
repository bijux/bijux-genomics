use std::path::Path;

use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::ValidateEffectiveParams;

pub const STAGE_ID: &str = "bam.validate";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[must_use]
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    bam_index: Option<&Path>,
    reference: Option<&Path>,
    out_dir: &Path,
) -> StagePlanV1 {
    let effective_params = ValidateEffectiveParams { strict: true };
    StagePlanV1 {
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
            outputs: vec![ArtifactRef {
                name: "validation_report".to_string(),
                path: out_dir.join("validation.json"),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "bam": bam,
            "bai": bam_index,
            "reference": reference,
            "strict": effective_params.strict,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .unwrap_or(serde_json::Value::Null),
        aux_images: std::collections::BTreeMap::new(),
    }
}
