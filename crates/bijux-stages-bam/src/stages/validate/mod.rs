use std::path::Path;

use bijux_core::{CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::ValidateEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Validate.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    bam_index: Option<&Path>,
    reference: Option<&Path>,
    out_dir: &Path,
) -> anyhow::Result<StagePlanV1> {
    let effective_params = ValidateEffectiveParams { strict: true };
    let outputs = super::audit_outputs(bijux_domain_bam::BamStage::Validate, out_dir);
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: super::args::samtools_validate_args(bam, &effective_params),
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
            "bai": bam_index,
            "reference": reference,
            "strict": effective_params.strict,
        }),
        effective_params: super::ensure_effective_params(
            serde_json::to_value(&effective_params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    super::ensure_required_outputs(plan, &["validation_report", "flagstat"])
}
